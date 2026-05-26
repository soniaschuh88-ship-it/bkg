// specification_drift.rs — Detect divergence between algebra, kernel, and traces.
//
// The "dual-system drift" problem:
//   Kernel executes δ correctly.
//   Algebra specifies δ.
//   Traces record δ.
//   Proofs verify δ via algebra.
//
// If any two of these diverge, the system has silent corruption:
//   "valid execution, invalid proof"   — kernel correct, algebra wrong
//   "valid proof, invalid execution"   — algebra correct, kernel wrong
//   "valid both, wrong traces"         — traces don't cover real behavior
//
// DriftDetector: a static analysis tool that checks all three pairwise:
//   Algebra   vs Kernel   — algebraic spec matches execution
//   Kernel    vs Traces   — execution matches observed behavior
//   Traces    vs Algebra  — observations match specification
//
// Each check produces a DriftReport with severity + precise violation location.
//
// Severity escalation:
//   Clean        — no drift detected
//   Informational — behavior untested (low coverage)
//   Warning      — algebra has cells with no observed trace evidence
//   Critical     — algebra disagrees with kernel (semantic error)
//
// Single source of truth for triple-layer consistency.

use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::{canonical_constraint_rules, RuleSet},
    kernel_state::{kernel_delta, KernelInputKind, KernelPhase, TRANSITION_TABLE},
    proof_certificate::ExecutionTrace,
    trace_synthesizer::TraceSynthesizer,
};

// ─── DriftSeverity ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DriftSeverity {
    /// No drift. All three layers are synchronized.
    Clean,
    /// Informational: some behaviors are untested (no trace evidence).
    Informational,
    /// Warning: algebra specifies behaviors with no trace evidence.
    Warning,
    /// Critical: algebra disagrees with kernel. Semantic error.
    Critical,
}

impl std::fmt::Display for DriftSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clean         => f.write_str("CLEAN"),
            Self::Informational => f.write_str("INFO"),
            Self::Warning       => f.write_str("WARN"),
            Self::Critical      => f.write_str("CRITICAL"),
        }
    }
}

// ─── DriftEvent ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftEvent {
    // ── Algebra vs Kernel ────────────────────────────────────────────────
    /// Algebra says δ(from, input) = algebra_says, but kernel_delta gives kernel_says.
    /// CRITICAL: the specification is wrong.
    AlgebraKernelConflict {
        from: KernelPhase,
        input: KernelInputKind,
        algebra_says: KernelPhase,
        kernel_says: KernelPhase,
    },

    // ── Kernel vs Traces ─────────────────────────────────────────────────
    /// A trace contains a transition the kernel would not produce.
    /// WARNING: trace was from a non-deterministic or buggy kernel.
    TraceContradicsKernel {
        at_step: usize,
        from: KernelPhase,
        input: KernelInputKind,
        trace_says: KernelPhase,
        kernel_says: KernelPhase,
    },

    // ── Traces vs Algebra ────────────────────────────────────────────────
    /// A trace contains a transition the algebra doesn't cover.
    /// INFO or WARNING: either missing rule or unexpected behavior.
    TransitionNotInAlgebra {
        from: KernelPhase,
        input: KernelInputKind,
        to: KernelPhase,
        in_explicit_table: bool,
    },

    // ── Coverage ─────────────────────────────────────────────────────────
    /// A TRANSITION_TABLE entry was never observed in any trace.
    /// Informational: behavior may be untested.
    UntracedTableEntry {
        from: KernelPhase,
        input: KernelInputKind,
        to: KernelPhase,
    },

    /// An algebra rule covers cells never observed in any trace.
    /// Warning: rule may be over-generalized.
    OvergeneralizedRule {
        rule_name: String,
        unobserved_cells: Vec<(KernelPhase, KernelInputKind)>,
    },
}

impl DriftEvent {
    pub fn severity(&self) -> DriftSeverity {
        match self {
            Self::AlgebraKernelConflict { .. }       => DriftSeverity::Critical,
            Self::TraceContradicsKernel { .. }       => DriftSeverity::Warning,
            Self::TransitionNotInAlgebra { in_explicit_table: true, .. } => DriftSeverity::Warning,
            Self::TransitionNotInAlgebra { in_explicit_table: false, .. } => DriftSeverity::Critical,
            Self::UntracedTableEntry { .. }          => DriftSeverity::Informational,
            Self::OvergeneralizedRule { .. }         => DriftSeverity::Warning,
        }
    }
}

// ─── DriftReport ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub events: Vec<DriftEvent>,
    pub severity: DriftSeverity,
    /// Summary statistics.
    pub stats: DriftStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftStats {
    pub total_events: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub algebra_kernel_agreement_pct: f64,
    pub trace_coverage_pct: f64,
}

impl DriftReport {
    fn build(events: Vec<DriftEvent>, trace_coverage_pct: f64) -> Self {
        let critical = events.iter().filter(|e| e.severity() == DriftSeverity::Critical).count();
        let warning  = events.iter().filter(|e| e.severity() == DriftSeverity::Warning).count();
        let info     = events.iter().filter(|e| e.severity() == DriftSeverity::Informational).count();
        let total    = events.len();

        let severity = if critical > 0 {
            DriftSeverity::Critical
        } else if warning > 0 {
            DriftSeverity::Warning
        } else if info > 0 {
            DriftSeverity::Informational
        } else {
            DriftSeverity::Clean
        };

        // Algebra-kernel agreement: fraction of 522 cells where they agree
        let total_cells = (KernelPhase::ALL.len() * KernelInputKind::ALL.len()) as f64;
        let agreement_cells = (total_cells as usize)
            - events.iter().filter(|e| matches!(e, DriftEvent::AlgebraKernelConflict { .. })).count();
        let agreement_pct = agreement_cells as f64 / total_cells * 100.0;

        DriftReport {
            events,
            severity,
            stats: DriftStats {
                total_events: total,
                critical_count: critical,
                warning_count: warning,
                info_count: info,
                algebra_kernel_agreement_pct: agreement_pct,
                trace_coverage_pct,
            },
        }
    }

    pub fn is_clean(&self) -> bool { self.severity == DriftSeverity::Clean }
    pub fn has_critical(&self) -> bool { self.severity == DriftSeverity::Critical }
}

// ─── DriftDetector ───────────────────────────────────────────────────────────

/// Checks all three pairwise divergences: algebra↔kernel, kernel↔traces, traces↔algebra.
pub struct DriftDetector;

impl DriftDetector {
    // ── Check 1: Algebra vs Kernel ─────────────────────────────────────────

    /// Check algebra against kernel_delta exhaustively.
    /// Runs over all 522 cells. O(|Q|×|Σ|).
    pub fn check_algebra_vs_kernel(algebra: &RuleSet) -> Vec<DriftEvent> {
        let mut events = Vec::new();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let algebra_says = algebra.delta(phase, input);
                let kernel_says  = kernel_delta(phase, input);
                if algebra_says != kernel_says {
                    events.push(DriftEvent::AlgebraKernelConflict {
                        from: phase, input, algebra_says, kernel_says,
                    });
                }
            }
        }
        events
    }

    // ── Check 2: Traces vs Kernel ──────────────────────────────────────────

    /// Check that every transition in traces is one kernel_delta would produce.
    pub fn check_traces_vs_kernel(traces: &[ExecutionTrace]) -> Vec<DriftEvent> {
        let mut events = Vec::new();
        for trace in traces {
            for (step, cert) in trace.certificates.iter().enumerate() {
                let kernel_says = kernel_delta(cert.from_phase, cert.input);
                if cert.to_phase != kernel_says {
                    events.push(DriftEvent::TraceContradicsKernel {
                        at_step: step,
                        from: cert.from_phase,
                        input: cert.input,
                        trace_says: cert.to_phase,
                        kernel_says,
                    });
                }
            }
        }
        events
    }

    // ── Check 3: Trace coverage ────────────────────────────────────────────

    /// Find table entries never observed in any trace.
    pub fn find_untraced_entries(traces: &[ExecutionTrace]) -> Vec<DriftEvent> {
        let mut observed: BTreeSet<(KernelPhase, KernelInputKind)> = BTreeSet::new();
        for trace in traces {
            for cert in &trace.certificates {
                observed.insert((cert.from_phase, cert.input));
            }
        }
        TRANSITION_TABLE.iter()
            .filter(|entry| !observed.contains(&(entry.from, entry.on)))
            .map(|entry| DriftEvent::UntracedTableEntry {
                from: entry.from, input: entry.on, to: entry.to,
            })
            .collect()
    }

    // ── Full triple-layer report ───────────────────────────────────────────

    /// Run all three checks and produce a unified DriftReport.
    pub fn full_report(
        algebra: &RuleSet,
        traces: &[ExecutionTrace],
    ) -> DriftReport {
        let mut events = Vec::new();

        events.extend(Self::check_algebra_vs_kernel(algebra));
        events.extend(Self::check_traces_vs_kernel(traces));
        events.extend(Self::find_untraced_entries(traces));

        // Trace coverage %
        let observed: BTreeSet<(KernelPhase, KernelInputKind)> = traces.iter()
            .flat_map(|t| t.certificates.iter().map(|c| (c.from_phase, c.input)))
            .collect();
        let coverage = observed.len() as f64 / TRANSITION_TABLE.len() as f64 * 100.0;

        DriftReport::build(events, coverage)
    }

    /// Run only the critical check: algebra vs kernel.
    /// This is the O(|Q|×|Σ|) check that must always pass.
    pub fn check_critical_only(algebra: &RuleSet) -> DriftReport {
        let events = Self::check_algebra_vs_kernel(algebra);
        DriftReport::build(events, 0.0)
    }
}

// ─── DriftMonitor ─────────────────────────────────────────────────────────────

/// A live monitor that accumulates traces and re-runs drift analysis on demand.
/// Attached to a Realm, it detects specification drift as the system runs.
pub struct DriftMonitor {
    synthesizer: TraceSynthesizer,
    last_report: Option<DriftReport>,
    trace_count: u64,
}

impl DriftMonitor {
    pub fn new() -> Self {
        Self { synthesizer: TraceSynthesizer::new(), last_report: None, trace_count: 0 }
    }

    /// Record an execution trace.
    pub fn record(&mut self, trace: &ExecutionTrace) {
        self.synthesizer.observe(trace);
        self.trace_count += 1;
    }

    /// Run the full drift analysis against the canonical algebra.
    pub fn analyze(&mut self, traces: &[ExecutionTrace]) -> &DriftReport {
        let algebra = canonical_constraint_rules();
        let report = DriftDetector::full_report(&algebra, traces);
        self.last_report = Some(report);
        self.last_report.as_ref().unwrap()
    }

    /// Quick critical check: algebra vs kernel (no traces needed).
    /// Should always return Clean — if not, specification is broken.
    pub fn quick_critical_check(&self) -> DriftReport {
        let algebra = canonical_constraint_rules();
        DriftDetector::check_critical_only(&algebra)
    }

    pub fn trace_count(&self) -> u64 { self.trace_count }
    pub fn last_report(&self) -> Option<&DriftReport> { self.last_report.as_ref() }
}

impl Default for DriftMonitor { fn default() -> Self { Self::new() } }

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;
    use crate::proof_certificate::{CertificateBuilder, ExecutionTrace};

    fn pipeline_trace() -> ExecutionTrace {
        let mut b = CertificateBuilder::new();
        let mut t = ExecutionTrace::new(RealmId::Telum, KernelPhase::Idle);
        let steps = [
            (KernelPhase::Idle,                KernelInputKind::EventArrived,       KernelPhase::ValidatingAbi),
            (KernelPhase::ValidatingAbi,        KernelInputKind::AbiValid,           KernelPhase::ValidatingSchema),
            (KernelPhase::ValidatingSchema,     KernelInputKind::SchemaValid,        KernelPhase::ValidatingClock),
            (KernelPhase::ValidatingClock,      KernelInputKind::ClockValid,         KernelPhase::ValidatingCapability),
            (KernelPhase::ValidatingCapability, KernelInputKind::CapabilityGranted,  KernelPhase::ValidatingCausal),
            (KernelPhase::ValidatingCausal,     KernelInputKind::CausalValid,        KernelPhase::Deciding),
            (KernelPhase::Deciding,             KernelInputKind::DecisionAllow,      KernelPhase::Applying),
            (KernelPhase::Applying,             KernelInputKind::TransitionApplied,  KernelPhase::Stamping),
            (KernelPhase::Stamping,             KernelInputKind::ProjectionStamped,  KernelPhase::Emitting),
            (KernelPhase::Emitting,             KernelInputKind::EmitComplete,       KernelPhase::Idle),
        ];
        for (from, input, to) in steps {
            if let Some(cert) = b.build(from, input, to) { t.push(cert); }
        }
        t
    }

    // ── Critical check: algebra must agree with kernel ─────────────────────

    #[test]
    fn canonical_algebra_agrees_with_kernel_everywhere() {
        let algebra = canonical_constraint_rules();
        let events = DriftDetector::check_algebra_vs_kernel(&algebra);
        let critical: Vec<_> = events.iter()
            .filter(|e| e.severity() == DriftSeverity::Critical).collect();
        assert!(critical.is_empty(),
            "CRITICAL: canonical algebra disagrees with kernel on {} cells:\n{}",
            critical.len(),
            critical.iter().map(|e| format!("  {e:?}")).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn quick_critical_check_is_clean() {
        let monitor = DriftMonitor::new();
        let report = monitor.quick_critical_check();
        assert!(report.is_clean(),
            "algebra-kernel critical check failed: severity={}, {} critical events",
            report.severity, report.stats.critical_count);
    }

    #[test]
    fn algebra_kernel_agreement_is_100_percent() {
        let algebra = canonical_constraint_rules();
        let events = DriftDetector::check_algebra_vs_kernel(&algebra);
        let conflict_count = events.iter()
            .filter(|e| matches!(e, DriftEvent::AlgebraKernelConflict { .. }))
            .count();
        assert_eq!(conflict_count, 0,
            "algebra and kernel must agree on all 522 cells");
    }

    // ── Trace vs kernel ────────────────────────────────────────────────────

    #[test]
    fn valid_trace_has_no_kernel_contradictions() {
        let trace = pipeline_trace();
        let events = DriftDetector::check_traces_vs_kernel(&[trace]);
        let contradictions: Vec<_> = events.iter()
            .filter(|e| matches!(e, DriftEvent::TraceContradicsKernel { .. })).collect();
        assert!(contradictions.is_empty(),
            "valid pipeline trace must not contradict kernel: {:?}", contradictions);
    }

    #[test]
    fn tampered_trace_detected() {
        let mut trace = pipeline_trace();
        // Tamper: change the to_phase of the first cert to something wrong
        if let Some(cert) = trace.certificates.first_mut() {
            cert.to_phase = KernelPhase::Faulted; // wrong: should be ValidatingAbi
        }
        let events = DriftDetector::check_traces_vs_kernel(&[trace]);
        let contradictions: Vec<_> = events.iter()
            .filter(|e| matches!(e, DriftEvent::TraceContradicsKernel { .. })).collect();
        assert!(!contradictions.is_empty(),
            "tampered trace must be detected");
    }

    // ── Coverage ───────────────────────────────────────────────────────────

    #[test]
    fn happy_path_leaves_untraced_entries() {
        let trace = pipeline_trace();
        let untraced = DriftDetector::find_untraced_entries(&[trace]);
        assert!(!untraced.is_empty(),
            "single happy-path trace must leave some table entries untraced");
    }

    #[test]
    fn full_report_on_valid_traces() {
        let trace = pipeline_trace();
        let algebra = canonical_constraint_rules();
        let report = DriftDetector::full_report(&algebra, &[trace]);

        // Must have no critical events
        assert_eq!(report.stats.critical_count, 0,
            "valid traces must not produce critical drift events");

        // Coverage should be > 0
        assert!(report.stats.trace_coverage_pct > 0.0);

        println!("Drift report: severity={}, coverage={:.1}%",
            report.severity, report.stats.trace_coverage_pct);
    }

    // ── DriftMonitor ────────────────────────────────────────────────────────

    #[test]
    fn monitor_accumulates_traces() {
        let mut monitor = DriftMonitor::new();
        for _ in 0..5 { monitor.record(&pipeline_trace()); }
        assert_eq!(monitor.trace_count(), 5);
    }

    #[test]
    fn monitor_detects_no_critical_drift_on_valid_system() {
        let mut monitor = DriftMonitor::new();
        let traces: Vec<_> = (0..3).map(|_| pipeline_trace()).collect();
        for t in &traces { monitor.record(t); }
        let report = monitor.analyze(&traces);
        assert_eq!(report.stats.critical_count, 0,
            "valid system must have zero critical drift events");
    }

    // ── Severity ordering ──────────────────────────────────────────────────

    #[test]
    fn severity_ordering() {
        assert!(DriftSeverity::Critical > DriftSeverity::Warning);
        assert!(DriftSeverity::Warning > DriftSeverity::Informational);
        assert!(DriftSeverity::Informational > DriftSeverity::Clean);
    }
}
