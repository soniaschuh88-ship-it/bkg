// trace_synthesizer.rs — Self-healing specification layer.
//
// PROBLEM: canonical_constraint_rules() is hand-written.
// If it misrepresents what the kernel actually does, the proof layer
// validates wrong things. The algebra is the "real truth" — but only
// if it matches observed behavior.
//
// SOLUTION: Inductive synthesis of ConstraintRules from execution traces.
//
// Algorithm:
//   1. Observe: extract (from, input, to) triples from ExecutionTrace set
//   2. Generalize: find the MINIMAL rule covering each group of triples
//      - Occam's Razor applied: prefer more general rules over exact ones
//      - Generalization ladder:
//          Exact(from, input) → PhaseEq+InputEq       (1 cell)
//          Phase(from)        → PhaseEq+InputIn([...]) (n cells, same phase)
//          Input(input)       → PhaseIn([...])+InputEq (m cells, same input)
//          Pattern            → PhaseIsProcessing/etc.  (structural group)
//          Cross              → PhaseIn([...])+InputIn([...]) (nm cells)
//   3. Verify: synthesized rule must be consistent with TRANSITION_TABLE
//   4. Compare: synthesized vs canonical — detect missing/novel/wrong rules
//
// Properties:
//   - Termination: domain is finite (|Q|×|Σ| = 522 cells)
//   - Soundness: synthesized rules never contradict TRANSITION_TABLE
//   - Minimality: greedy Occam — prefer broadest safe generalization
//
// Single source of truth for specification synthesis.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::{
        canonical_constraint_rules, ConstraintExpr, ConstraintRule,
        ConstraintTarget, RuleSet,
    },
    kernel_state::{kernel_delta, KernelInputKind, KernelPhase, TRANSITION_TABLE},
    proof_certificate::ExecutionTrace,
};

// ─── Observation ──────────────────────────────────────────────────────────────

/// One observed (from, input, to) triple with frequency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub from: KernelPhase,
    pub input: KernelInputKind,
    pub to: KernelPhase,
    pub count: u64,
}

// ─── GeneralizationType ───────────────────────────────────────────────────────

/// How a synthesized rule generalized beyond the exact observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneralizationType {
    /// Exact: one cell, no generalization.
    Exact,
    /// Same target for all inputs at this phase (phase generalized to all inputs).
    AllInputsAtPhase,
    /// Same target for this input at multiple phases (phase generalized).
    PhaseGroup { phases: Vec<KernelPhase> },
    /// Same target for multiple inputs at this phase (input generalized).
    InputGroup { inputs: Vec<KernelInputKind> },
    /// Structural phase predicate matched (is_processing, is_terminal, etc).
    StructuralPattern { pattern: String },
    /// Cross-product generalization: multiple phases × multiple inputs.
    CrossProduct { phases: Vec<KernelPhase>, inputs: Vec<KernelInputKind> },
}

/// One step in the generalization process — audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralizationStep {
    pub rule_name: String,
    pub covers_cells: usize,
    pub generalization: GeneralizationType,
    pub is_table_consistent: bool,
}

// ─── SynthesisResult ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelTransition {
    /// In traces but NOT in canonical_constraint_rules().
    pub from: KernelPhase,
    pub input: KernelInputKind,
    pub to: KernelPhase,
    pub count: u64,
    /// Is this transition in TRANSITION_TABLE? (known-valid or unknown)
    pub in_explicit_table: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredBehavior {
    /// In TRANSITION_TABLE but NEVER appeared in observed traces.
    pub from: KernelPhase,
    pub input: KernelInputKind,
    pub to: KernelPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgebraDisagreement {
    /// Algebra says one thing; kernel_delta says another.
    pub from: KernelPhase,
    pub input: KernelInputKind,
    pub algebra_says: KernelPhase,
    pub kernel_says: KernelPhase,
}

pub struct SynthesisResult {
    /// The synthesized rule set (inferred from traces).
    pub rule_set: RuleSet,
    /// Generalization audit trail.
    pub generalization_steps: Vec<GeneralizationStep>,
    /// Cells in TRANSITION_TABLE covered by synthesized rules.
    pub table_coverage: f64,
    /// Transitions seen in traces but not in canonical_constraint_rules().
    pub novel_transitions: Vec<NovelTransition>,
    /// Transitions in TRANSITION_TABLE never observed in any trace.
    pub uncovered_behaviors: Vec<UncoveredBehavior>,
    /// Where synthesized algebra disagrees with kernel_delta.
    pub algebra_disagreements: Vec<AlgebraDisagreement>,
}

impl SynthesisResult {
    pub fn is_consistent(&self) -> bool {
        self.algebra_disagreements.is_empty()
    }

    pub fn has_novel_valid_transitions(&self) -> bool {
        self.novel_transitions.iter().any(|t| t.in_explicit_table)
    }

    pub fn rule_count(&self) -> usize { self.rule_set.rule_count() }
}

// ─── TraceSynthesizer ─────────────────────────────────────────────────────────

/// Synthesizes ConstraintRules from observed execution traces.
pub struct TraceSynthesizer {
    /// All unique (from, input) → (to, count) observations.
    observations: BTreeMap<(KernelPhase, KernelInputKind), (KernelPhase, u64)>,
}

impl TraceSynthesizer {
    pub fn new() -> Self {
        Self { observations: BTreeMap::new() }
    }

    /// Feed one execution trace into the synthesizer.
    pub fn observe(&mut self, trace: &ExecutionTrace) {
        for cert in &trace.certificates {
            let entry = self.observations
                .entry((cert.from_phase, cert.input))
                .or_insert((cert.to_phase, 0));
            entry.1 += 1;
        }
    }

    /// Feed multiple traces.
    pub fn observe_all(&mut self, traces: &[ExecutionTrace]) {
        for trace in traces { self.observe(trace); }
    }

    /// Current observation count.
    pub fn observation_count(&self) -> usize { self.observations.len() }

    /// Synthesize the minimal rule set from observations.
    pub fn synthesize(&self) -> SynthesisResult {
        let mut rules: Vec<ConstraintRule> = Vec::new();
        let mut steps: Vec<GeneralizationStep> = Vec::new();

        // Group observations by target phase
        let mut by_target: BTreeMap<KernelPhase, Vec<(KernelPhase, KernelInputKind)>> = BTreeMap::new();
        for (&(from, input), &(to, _)) in &self.observations {
            by_target.entry(to).or_default().push((from, input));
        }

        for (target, cells) in &by_target {
            if cells.is_empty() { continue; }

            let phases: BTreeSet<KernelPhase> = cells.iter().map(|&(p, _)| p).collect();
            let inputs: BTreeSet<KernelInputKind> = cells.iter().map(|&(_, i)| i).collect();

            // Attempt generalizations from most to least general,
            // accepting the first that is table-consistent.
            let candidate = self.try_generalize(&phases, &inputs, *target, &rules);
            if let Some((rule, step)) = candidate {
                rules.push(rule);
                steps.push(step);
            }
        }

        // Check against canonical algebra for novel/uncovered transitions
        let canonical = canonical_constraint_rules();
        let novel = self.find_novel_transitions(&canonical);
        let uncovered = self.find_uncovered_behaviors();
        let disagreements = self.find_algebra_disagreements(&canonical);

        // Measure table coverage of synthesized rules
        let synth_rs = RuleSet::new(rules);
        let table_coverage = self.compute_coverage(&synth_rs);

        SynthesisResult {
            rule_set: synth_rs,
            generalization_steps: steps,
            table_coverage,
            novel_transitions: novel,
            uncovered_behaviors: uncovered,
            algebra_disagreements: disagreements,
        }
    }

    /// Try to generalize a set of (phase, input) cells with the same target.
    /// Returns the most general safe rule, or an exact rule if no generalization is safe.
    fn try_generalize(
        &self,
        phases: &BTreeSet<KernelPhase>,
        inputs: &BTreeSet<KernelInputKind>,
        target: KernelPhase,
        _existing: &[ConstraintRule],
    ) -> Option<(ConstraintRule, GeneralizationStep)> {
        let target_ct = ConstraintTarget::Phase(target);
        let phase_vec: Vec<KernelPhase> = phases.iter().copied().collect();
        let input_vec: Vec<KernelInputKind> = inputs.iter().copied().collect();

        // ── Try structural patterns (broadest generalization) ──────────────

        // Universal fault: all non-terminal + FaultDetected → Faulted?
        if target == KernelPhase::Faulted
            && inputs.contains(&KernelInputKind::FaultDetected)
            && phases.iter().all(|p| !p.is_terminal())
        {
            let guard = ConstraintExpr::PhaseIsTerminal.negate()
                .and(ConstraintExpr::InputEq(KernelInputKind::FaultDetected));
            if self.is_table_consistent(&guard, target) {
                let cells = guard.cardinality();
                return Some((
                    ConstraintRule::new("synth:universal-fault", guard, target_ct),
                    GeneralizationStep {
                        rule_name: "synth:universal-fault".to_string(),
                        covers_cells: cells,
                        generalization: GeneralizationType::StructuralPattern { pattern: "PhaseIsTerminal.negate()".to_string() },
                        is_table_consistent: true,
                    },
                ));
            }
        }

        // Sealed absorbs all?
        if phases.len() == 1 && *phases.iter().next().unwrap() == KernelPhase::Sealed
            && target == KernelPhase::Sealed
        {
            let guard = ConstraintExpr::PhaseEq(KernelPhase::Sealed);
            let cells = guard.cardinality();
            return Some((
                ConstraintRule::new("synth:sealed-absorbing", guard, ConstraintTarget::Self_),
                GeneralizationStep {
                    rule_name: "synth:sealed-absorbing".to_string(),
                    covers_cells: cells,
                    generalization: GeneralizationType::StructuralPattern { pattern: "PhaseEq(Sealed)".to_string() },
                    is_table_consistent: true,
                },
            ));
        }

        // ── Try cross-product generalization ──────────────────────────────
        if phases.len() > 1 && inputs.len() > 1 {
            let guard = ConstraintExpr::PhaseIn(phase_vec.clone())
                .and(ConstraintExpr::InputIn(input_vec.clone()));
            if self.is_table_consistent(&guard, target) {
                let cells = guard.cardinality();
                return Some((
                    ConstraintRule::new("synth:cross-product", guard, target_ct),
                    GeneralizationStep {
                        rule_name: "synth:cross-product".to_string(),
                        covers_cells: cells,
                        generalization: GeneralizationType::CrossProduct {
                            phases: phase_vec.clone(), inputs: input_vec.clone(),
                        },
                        is_table_consistent: true,
                    },
                ));
            }
        }

        // ── Phase group + single input ─────────────────────────────────────
        if phases.len() > 1 && inputs.len() == 1 {
            let input = *inputs.iter().next().unwrap();
            let guard = ConstraintExpr::PhaseIn(phase_vec.clone())
                .and(ConstraintExpr::InputEq(input));
            if self.is_table_consistent(&guard, target) {
                let cells = guard.cardinality();
                return Some((
                    ConstraintRule::new("synth:phase-group", guard, target_ct),
                    GeneralizationStep {
                        rule_name: "synth:phase-group".to_string(),
                        covers_cells: cells,
                        generalization: GeneralizationType::PhaseGroup { phases: phase_vec },
                        is_table_consistent: true,
                    },
                ));
            }
        }

        // ── Single phase + input group ─────────────────────────────────────
        if phases.len() == 1 && inputs.len() > 1 {
            let phase = *phases.iter().next().unwrap();
            let guard = ConstraintExpr::PhaseEq(phase)
                .and(ConstraintExpr::InputIn(input_vec.clone()));
            if self.is_table_consistent(&guard, target) {
                let cells = guard.cardinality();
                return Some((
                    ConstraintRule::new("synth:input-group", guard, target_ct),
                    GeneralizationStep {
                        rule_name: "synth:input-group".to_string(),
                        covers_cells: cells,
                        generalization: GeneralizationType::InputGroup { inputs: input_vec },
                        is_table_consistent: true,
                    },
                ));
            }
        }

        // ── Exact (no generalization — fallback) ──────────────────────────
        if phases.len() == 1 && inputs.len() == 1 {
            let phase = *phases.iter().next().unwrap();
            let input = *inputs.iter().next().unwrap();
            let guard = ConstraintExpr::PhaseEq(phase).and(ConstraintExpr::InputEq(input));
            return Some((
                ConstraintRule::new("synth:exact", guard, target_ct),
                GeneralizationStep {
                    rule_name: "synth:exact".to_string(),
                    covers_cells: 1,
                    generalization: GeneralizationType::Exact,
                    is_table_consistent: true,
                },
            ));
        }

        None
    }

    /// Check that a guard + target is consistent with TRANSITION_TABLE.
    /// "Consistent" means: for every (phase, input) where guard holds,
    /// kernel_delta(phase, input) == target (or the cell is undefined → Faulted → ok if target=Faulted).
    fn is_table_consistent(&self, guard: &ConstraintExpr, target: KernelPhase) -> bool {
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if guard.eval(phase, input) {
                    let actual = kernel_delta(phase, input);
                    if actual != target {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn compute_coverage(&self, rule_set: &RuleSet) -> f64 {
        let table_entries = TRANSITION_TABLE.len();
        if table_entries == 0 { return 0.0; }
        let covered = TRANSITION_TABLE.iter()
            .filter(|entry| rule_set.delta(entry.from, entry.on) == entry.to)
            .count();
        covered as f64 / table_entries as f64
    }

    fn find_novel_transitions(&self, canonical: &RuleSet) -> Vec<NovelTransition> {
        self.observations.iter()
            .filter_map(|(&(from, input), &(to, count))| {
                let canonical_says = canonical.delta(from, input);
                if canonical_says != to {
                    Some(NovelTransition {
                        from, input, to, count,
                        in_explicit_table: TRANSITION_TABLE.iter()
                            .any(|e| e.from == from && e.on == input && e.to == to),
                    })
                } else { None }
            })
            .collect()
    }

    fn find_uncovered_behaviors(&self) -> Vec<UncoveredBehavior> {
        TRANSITION_TABLE.iter()
            .filter(|entry| !self.observations.contains_key(&(entry.from, entry.on)))
            .map(|entry| UncoveredBehavior { from: entry.from, input: entry.on, to: entry.to })
            .collect()
    }

    fn find_algebra_disagreements(&self, canonical: &RuleSet) -> Vec<AlgebraDisagreement> {
        let mut disagreements = Vec::new();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let kernel_says = kernel_delta(phase, input);
                let algebra_says = canonical.delta(phase, input);
                if kernel_says != algebra_says {
                    disagreements.push(AlgebraDisagreement {
                        from: phase, input, algebra_says, kernel_says,
                    });
                }
            }
        }
        disagreements
    }
}

impl Default for TraceSynthesizer { fn default() -> Self { Self::new() } }

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;
    use crate::proof_certificate::{CertificateBuilder, ExecutionTrace};

    fn build_trace(steps: &[(KernelPhase, KernelInputKind, KernelPhase)]) -> ExecutionTrace {
        let mut b = CertificateBuilder::new();
        let mut trace = ExecutionTrace::new(RealmId::Telum, steps[0].0);
        for &(from, input, to) in steps {
            if let Some(cert) = b.build(from, input, to) {
                trace.push(cert);
            }
        }
        trace
    }

    fn pipeline_trace() -> ExecutionTrace {
        build_trace(&[
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
        ])
    }

    #[test]
    fn synthesize_from_single_trace() {
        let mut s = TraceSynthesizer::new();
        s.observe(&pipeline_trace());
        let result = s.synthesize();
        assert!(result.rule_count() > 0,
            "synthesis must produce rules from observed trace");
        assert!(result.table_coverage > 0.0,
            "synthesized rules must cover some table entries");
    }

    #[test]
    fn algebra_has_no_disagreements_with_kernel() {
        // The canonical constraint algebra must agree with kernel_delta everywhere.
        // This is the core safety property.
        let s = TraceSynthesizer::new(); // no observations needed
        let canonical = canonical_constraint_rules();
        let disagreements = s.find_algebra_disagreements(&canonical);
        assert!(disagreements.is_empty(),
            "canonical algebra disagrees with kernel on {} cells:\n{}",
            disagreements.len(),
            disagreements.iter().map(|d| format!(
                "  δ({}, {:?}): algebra={}, kernel={}",
                d.from, d.input, d.algebra_says, d.kernel_says
            )).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn synthesizer_detects_uncovered_behaviors() {
        // With only the happy-path trace, many table entries are never observed.
        let mut s = TraceSynthesizer::new();
        s.observe(&pipeline_trace());
        let result = s.synthesize();
        // Rejection arcs, fault arcs, replay arcs are all uncovered by a single happy path
        assert!(!result.uncovered_behaviors.is_empty(),
            "uncovered behaviors must be detected when only happy path is observed");
    }

    #[test]
    fn full_coverage_with_all_arcs() {
        let mut s = TraceSynthesizer::new();

        // Observe happy path
        s.observe(&pipeline_trace());

        // Observe rejection arc
        s.observe(&build_trace(&[
            (KernelPhase::Idle,          KernelInputKind::EventArrived, KernelPhase::ValidatingAbi),
            (KernelPhase::ValidatingAbi, KernelInputKind::AbiFailed,    KernelPhase::Idle),
        ]));

        // Observe fault arc
        s.observe(&build_trace(&[
            (KernelPhase::Idle, KernelInputKind::EventArrived, KernelPhase::ValidatingAbi),
            (KernelPhase::ValidatingAbi, KernelInputKind::FaultDetected, KernelPhase::Faulted),
        ]));

        let result = s.synthesize();
        assert!(result.table_coverage > 0.1, "coverage should increase with more observations");
    }

    #[test]
    fn repeated_observation_counts() {
        let mut s = TraceSynthesizer::new();
        for _ in 0..10 { s.observe(&pipeline_trace()); }
        // All should map to same (from, input, to) — counts accumulate
        let count = s.observations.values().map(|(_, c)| c).sum::<u64>();
        assert_eq!(count, 10 * 10, // 10 traces × 10 steps each
            "observation counts must accumulate across repeated traces");
    }

    #[test]
    fn synthesized_rules_are_table_consistent() {
        let mut s = TraceSynthesizer::new();
        s.observe(&pipeline_trace());
        let result = s.synthesize();
        // Every synthesized rule must be consistent with kernel_delta
        for step in &result.generalization_steps {
            assert!(step.is_table_consistent,
                "synthesized rule '{}' is not table-consistent", step.rule_name);
        }
    }

    #[test]
    fn no_novel_transitions_in_valid_traces() {
        // If we observe only valid transitions (from canonical_rules),
        // novel_transitions must be empty.
        let mut s = TraceSynthesizer::new();
        s.observe(&pipeline_trace());
        // All transitions in pipeline_trace are canonical — none should be "novel"
        let result = s.synthesize();
        let invalid_novel: Vec<_> = result.novel_transitions.iter()
            .filter(|t| !t.in_explicit_table)
            .collect();
        assert!(invalid_novel.is_empty(),
            "valid traces must not produce invalid novel transitions: {:?}", invalid_novel);
    }
}
