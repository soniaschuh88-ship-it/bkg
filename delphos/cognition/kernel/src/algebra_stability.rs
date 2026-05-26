// algebra_stability.rs — Meta-invariants: rules about rules.
//
// PROBLEM: The synthesis loop has no bounds on spec evolution.
// Without constraints:
//   - Synthesis can remove structural rules and replace with exact pairs (spec regression)
//   - Coverage can grow but expressiveness collapse (spec hollowing)
//   - Pinned rules (universal-fault, sealed-absorbing) can be weakened
//
// SOLUTION: AlgebraInvariant — a named check on a RuleSet.
//
// Invariants form two categories:
//
//   HARD invariants (violations → synthesis rejected):
//     KernelAlignment    — algebra must agree with kernel on all 522 cells
//     PipelineAcyclic    — processing phases must be a DAG
//     NoConflicts        — no two rules produce different targets for same cell
//     PinnedRulesPresent — anchored rules must exist in the set
//
//   SOFT invariants (violations → warning, not rejection):
//     MinimumCoverage    — must explain at least N% of table entries
//     ExpressionFloor    — expressiveness above minimum entropy floor
//
// PinnedRuleSet: wraps a RuleSet with a set of anchored rule names.
// Synthesis cannot produce a rule set where pinned rules are absent or weakened.
//
// SynthesisCycleGuard: enforces bounds ACROSS multiple synthesis cycles.
// Tracks the history of rule sets and checks:
//   - Monotone coverage: coverage(t+1) >= coverage(t)
//   - Expressiveness bound: expressiveness(t+1) >= expressiveness(t) - epsilon
//   - Pinned rule preservation: all pinned rules present in t+1
//
// Single source of truth for meta-invariants.

use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::{canonical_constraint_rules, RuleSet},
    kernel_state::{kernel_delta, KernelInputKind, KernelPhase},
    specification_entropy::{EntropyFloor, EntropyMeasure, SpecificationEntropy},
};

// ─── InvariantResult ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantResult {
    Passed,
    Failed { reason: String },
}

impl InvariantResult {
    pub fn passed(&self) -> bool { *self == Self::Passed }
    pub fn failed(&self) -> bool { matches!(self, Self::Failed { .. }) }
    pub fn ok() -> Self { Self::Passed }
    pub fn fail(reason: impl Into<String>) -> Self { Self::Failed { reason: reason.into() } }
}

// ─── InvariantSeverity ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantSeverity {
    /// Warning: violation logged but synthesis not blocked.
    Soft,
    /// Hard: violation → synthesis result rejected.
    Hard,
}

// ─── AlgebraInvariant ────────────────────────────────────────────────────────

/// A named meta-invariant on a RuleSet.
#[derive(Debug, Clone)]
pub struct AlgebraInvariant {
    pub name: &'static str,
    pub severity: InvariantSeverity,
}

impl AlgebraInvariant {
    pub fn check(&self, rule_set: &RuleSet) -> InvariantResult {
        match self.name {
            "kernel-alignment" => check_kernel_alignment(rule_set),
            "pipeline-acyclic" => check_pipeline_acyclic(rule_set),
            "no-conflicts"     => check_no_conflicts(rule_set),
            "minimum-coverage" => check_minimum_coverage(rule_set, 0.5),
            "expression-floor" => check_expression_floor(rule_set, &EntropyFloor::development()),
            name => InvariantResult::fail(format!("unknown invariant: {name}")),
        }
    }
}

// ─── Hard invariant implementations ──────────────────────────────────────────

fn check_kernel_alignment(rs: &RuleSet) -> InvariantResult {
    let mut violations = 0;
    for &phase in KernelPhase::ALL {
        for &input in KernelInputKind::ALL {
            if rs.delta(phase, input) != kernel_delta(phase, input) {
                violations += 1;
            }
        }
    }
    if violations == 0 {
        InvariantResult::Passed
    } else {
        InvariantResult::fail(format!(
            "algebra disagrees with kernel on {violations} of {} cells",
            KernelPhase::ALL.len() * KernelInputKind::ALL.len()
        ))
    }
}

fn check_pipeline_acyclic(rs: &RuleSet) -> InvariantResult {
    let proof = rs.prove_pipeline_acyclic();
    if proof.is_acyclic {
        InvariantResult::Passed
    } else {
        InvariantResult::fail(format!(
            "pipeline has {} backward arcs: {:?}",
            proof.cycles.len(), proof.cycles
        ))
    }
}

fn check_no_conflicts(rs: &RuleSet) -> InvariantResult {
    let report = rs.verify_consistency();
    if report.conflicts.is_empty() {
        InvariantResult::Passed
    } else {
        InvariantResult::fail(format!(
            "{} conflicting rule pairs detected",
            report.conflicts.len()
        ))
    }
}

fn check_minimum_coverage(rs: &RuleSet, threshold: f64) -> InvariantResult {
    use crate::kernel_state::TRANSITION_TABLE;
    let covered = TRANSITION_TABLE.iter()
        .filter(|e| rs.delta(e.from, e.on) == e.to)
        .count();
    let actual = covered as f64 / TRANSITION_TABLE.len() as f64;
    if actual >= threshold {
        InvariantResult::Passed
    } else {
        InvariantResult::fail(format!(
            "coverage {:.1}% below threshold {:.1}%",
            actual * 100.0, threshold * 100.0
        ))
    }
}

fn check_expression_floor(rs: &RuleSet, floor: &EntropyFloor) -> InvariantResult {
    let m = SpecificationEntropy::measure(rs);
    if m.is_above_floor(floor) {
        InvariantResult::Passed
    } else {
        InvariantResult::fail(format!(
            "below entropy floor: {}", m.report()
        ))
    }
}

// ─── Standard invariant set ───────────────────────────────────────────────────

/// The standard set of algebra invariants for DELPHOS.
pub fn standard_invariants() -> Vec<AlgebraInvariant> {
    vec![
        AlgebraInvariant { name: "kernel-alignment", severity: InvariantSeverity::Hard },
        AlgebraInvariant { name: "pipeline-acyclic",  severity: InvariantSeverity::Hard },
        AlgebraInvariant { name: "no-conflicts",      severity: InvariantSeverity::Hard },
        AlgebraInvariant { name: "minimum-coverage",  severity: InvariantSeverity::Soft },
        AlgebraInvariant { name: "expression-floor",  severity: InvariantSeverity::Soft },
    ]
}

// ─── InvariantCheckReport ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantCheckResult {
    pub invariant_name: String,
    pub severity: InvariantSeverity,
    pub result: InvariantResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantCheckReport {
    pub results: Vec<InvariantCheckResult>,
    pub hard_violations: usize,
    pub soft_violations: usize,
    pub is_acceptable: bool, // true if no hard violations
}

impl InvariantCheckReport {
    pub fn run(rule_set: &RuleSet, invariants: &[AlgebraInvariant]) -> Self {
        let results: Vec<InvariantCheckResult> = invariants.iter()
            .map(|inv| InvariantCheckResult {
                invariant_name: inv.name.to_string(),
                severity: inv.severity,
                result: inv.check(rule_set),
            })
            .collect();

        let hard = results.iter()
            .filter(|r| r.severity == InvariantSeverity::Hard && r.result.failed())
            .count();
        let soft = results.iter()
            .filter(|r| r.severity == InvariantSeverity::Soft && r.result.failed())
            .count();

        Self { results, hard_violations: hard, soft_violations: soft, is_acceptable: hard == 0 }
    }

    pub fn all_hard_passed(&self) -> bool { self.hard_violations == 0 }
    pub fn all_passed(&self) -> bool { self.hard_violations == 0 && self.soft_violations == 0 }
}

// ─── PinnedRuleSet ────────────────────────────────────────────────────────────

/// A RuleSet with named rules that CANNOT be removed or weakened by synthesis.
///
/// Pinned rules are the semantic anchors — invariant facts about the system
/// that must survive any synthesis cycle. If synthesis produces a rule set
/// where a pinned rule is absent (or its coverage is reduced), the result
/// is rejected.
pub struct PinnedRuleSet {
    pub rule_set: RuleSet,
    pinned_names: BTreeSet<&'static str>,
    /// Minimum coverage for each pinned rule (cells it must cover).
    pinned_coverage: std::collections::BTreeMap<&'static str, usize>,
}

impl PinnedRuleSet {
    /// Create from the canonical rule set with default pinned rules.
    pub fn from_canonical() -> Self {
        let rs = canonical_constraint_rules();
        let pinned = ["sealed-absorbs-all", "faulted-absorbs-non-recovery",
                      "universal-fault", "pipeline-advance", "validation-rejection"]
            .iter().copied().collect::<BTreeSet<_>>();

        // Record current coverage of each pinned rule as the minimum baseline
        let pinned_coverage = pinned.iter().filter_map(|&name| {
            rs.rules.iter().find(|r| r.name == name)
                .map(|r| (name, r.domain().len()))
        }).collect();

        Self { rule_set: rs, pinned_names: pinned, pinned_coverage }
    }

    pub fn new(rule_set: RuleSet, pinned: BTreeSet<&'static str>) -> Self {
        let pinned_coverage = pinned.iter().filter_map(|&name| {
            rule_set.rules.iter().find(|r| r.name == name)
                .map(|r| (name, r.domain().len()))
        }).collect();
        Self { rule_set, pinned_names: pinned, pinned_coverage }
    }

    /// Check that a candidate rule set preserves all pinned rules.
    pub fn check_candidate(&self, candidate: &RuleSet) -> PinnedRuleCheck {
        let mut missing = Vec::new();
        let mut weakened = Vec::new();

        for &name in &self.pinned_names {
            match candidate.rules.iter().find(|r| r.name == name) {
None => missing.push(name.to_string()),
                Some(rule) => {
                    let current_coverage = rule.domain().len();
                    let required = self.pinned_coverage.get(name).copied().unwrap_or(0);
                    if current_coverage < required {
                        weakened.push(PinnedWeakening {
                            rule_name: name.to_string(),
                            required_cells: required,
                            actual_cells: current_coverage,
                        });
                    }
                }
            }
        }

        PinnedRuleCheck {
            is_valid: missing.is_empty() && weakened.is_empty(),
            missing_rules: missing,
            weakened_rules: weakened,
        }
    }

    pub fn pinned_names(&self) -> &BTreeSet<&'static str> { &self.pinned_names }
    pub fn pinned_count(&self) -> usize { self.pinned_names.len() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedRuleCheck {
    pub is_valid: bool,
    pub missing_rules: Vec<String>,
    pub weakened_rules: Vec<PinnedWeakening>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedWeakening {
    pub rule_name: String,
    pub required_cells: usize,
    pub actual_cells: usize,
}

// ─── SynthesisCycleGuard ──────────────────────────────────────────────────────

/// Enforces stability invariants ACROSS multiple synthesis cycles.
///
/// Tracks:
///   - History of rule sets and their entropy measures
///   - Monotone coverage requirement
///   - Expressiveness bound (can't degrade by more than epsilon)
///   - Pinned rule preservation across all cycles
pub struct SynthesisCycleGuard {
    pub invariants: Vec<AlgebraInvariant>,
    pinned: PinnedRuleSet,
    history: Vec<CycleSnapshot>,
    /// Maximum allowed expressiveness degradation per cycle.
    pub expressiveness_tolerance: f64,
    /// Maximum allowed coverage decrease per cycle (0 = must not decrease).
    pub coverage_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleSnapshot {
    pub cycle_index: u64,
    pub rule_count: usize,
    pub entropy: EntropyMeasure,
    pub hard_violations: usize,
    pub soft_violations: usize,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleAcceptance {
    pub accepted: bool,
    pub hard_violations: Vec<String>,
    pub soft_violations: Vec<String>,
    pub pinned_check: PinnedRuleCheck,
    pub entropy_delta: Option<crate::specification_entropy::EntropyCycleDelta>,
    pub cycle_index: u64,
}

impl CycleAcceptance {
    pub fn reasons_for_rejection(&self) -> Vec<String> {
        let mut reasons = self.hard_violations.clone();
        if !self.pinned_check.is_valid {
            reasons.push(format!(
                "pinned rules missing: {:?}, weakened: {:?}",
                self.pinned_check.missing_rules, self.pinned_check.weakened_rules
            ));
        }
        if let Some(delta) = &self.entropy_delta {
            if delta.is_degrading() {
                reasons.push(format!(
                    "expressiveness degraded by {:.3}", delta.expressiveness_delta.abs()
                ));
            }
        }
        reasons
    }
}

impl SynthesisCycleGuard {
    pub fn new() -> Self {
        Self {
            invariants: standard_invariants(),
            pinned: PinnedRuleSet::from_canonical(),
            history: Vec::new(),
            expressiveness_tolerance: 0.05, // allow 5% degradation per cycle
            coverage_tolerance: 0.0,        // coverage must never decrease
        }
    }

    /// Evaluate a candidate rule set for a new synthesis cycle.
    /// Returns CycleAcceptance with full audit trail.
    pub fn evaluate(&mut self, candidate: &RuleSet) -> CycleAcceptance {
        let cycle_index = self.history.len() as u64 + 1;

        // Run invariants
        let report = InvariantCheckReport::run(candidate, &self.invariants);
        let hard_violations: Vec<String> = report.results.iter()
            .filter(|r| r.severity == InvariantSeverity::Hard && r.result.failed())
            .map(|r| match &r.result {
                InvariantResult::Failed { reason } => format!("[{}] {}", r.invariant_name, reason),
                _ => unreachable!(),
            })
            .collect();
        let soft_violations: Vec<String> = report.results.iter()
            .filter(|r| r.severity == InvariantSeverity::Soft && r.result.failed())
            .map(|r| match &r.result {
                InvariantResult::Failed { reason } => format!("[{}] {}", r.invariant_name, reason),
                _ => unreachable!(),
            })
            .collect();

        // Check pinned rules
        let pinned_check = self.pinned.check_candidate(candidate);

        // Check entropy delta
        let new_entropy = SpecificationEntropy::measure(candidate);
        let entropy_delta = self.history.last().map(|prev| {
            SpecificationEntropy::measure_cycle_delta(&prev.entropy, &new_entropy)
        });

        // Determine coverage monotonicity
        let coverage_ok = self.history.last()
            .map(|prev| new_entropy.table_coverage >= prev.entropy.table_coverage - self.coverage_tolerance)
            .unwrap_or(true);

        // Determine expressiveness bound
        let expressiveness_ok = entropy_delta.as_ref()
            .map(|d| d.expressiveness_delta >= -self.expressiveness_tolerance)
            .unwrap_or(true);

        let accepted = hard_violations.is_empty()
            && pinned_check.is_valid
            && coverage_ok
            && expressiveness_ok;

        let snapshot = CycleSnapshot {
            cycle_index,
            rule_count: candidate.rule_count(),
            entropy: new_entropy,
            hard_violations: hard_violations.len(),
            soft_violations: soft_violations.len(),
            accepted,
        };
        self.history.push(snapshot);

        CycleAcceptance {
            accepted, hard_violations, soft_violations,
            pinned_check, entropy_delta, cycle_index,
        }
    }

    pub fn cycle_count(&self) -> usize { self.history.len() }
    pub fn acceptance_rate(&self) -> f64 {
        if self.history.is_empty() { return 0.0; }
        let accepted = self.history.iter().filter(|s| s.accepted).count();
        accepted as f64 / self.history.len() as f64
    }
    pub fn history(&self) -> &[CycleSnapshot] { &self.history }
}

impl Default for SynthesisCycleGuard { fn default() -> Self { Self::new() } }

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Hard invariants on canonical spec ─────────────────────────────────

    #[test]
    fn canonical_passes_kernel_alignment() {
        let rs = canonical_constraint_rules();
        let inv = AlgebraInvariant { name: "kernel-alignment", severity: InvariantSeverity::Hard };
        assert!(inv.check(&rs).passed(),
            "canonical spec must be kernel-aligned");
    }

    #[test]
    fn canonical_passes_pipeline_acyclic() {
        let rs = canonical_constraint_rules();
        let inv = AlgebraInvariant { name: "pipeline-acyclic", severity: InvariantSeverity::Hard };
        assert!(inv.check(&rs).passed(),
            "canonical spec must have acyclic pipeline");
    }

    #[test]
    fn canonical_passes_no_conflicts() {
        let rs = canonical_constraint_rules();
        let inv = AlgebraInvariant { name: "no-conflicts", severity: InvariantSeverity::Hard };
        assert!(inv.check(&rs).passed(),
            "canonical spec must have no conflicts");
    }

    #[test]
    fn canonical_passes_all_standard_invariants() {
        let rs = canonical_constraint_rules();
        let report = InvariantCheckReport::run(&rs, &standard_invariants());
        assert!(report.all_hard_passed(),
            "canonical spec must pass all hard invariants: {:?}",
            report.results.iter()
                .filter(|r| r.result.failed())
                .map(|r| (&r.invariant_name, &r.result))
                .collect::<Vec<_>>());
    }

    // ── Pinned rules ───────────────────────────────────────────────────────

    #[test]
    fn canonical_has_all_pinned_rules() {
        let pinned = PinnedRuleSet::from_canonical();
        let check = pinned.check_candidate(&pinned.rule_set);
        assert!(check.is_valid,
            "canonical spec must satisfy its own pinned rules: {:?}", check);
    }

    #[test]
    fn removing_pinned_rule_fails_check() {
        let pinned = PinnedRuleSet::from_canonical();
        // Create a candidate with the universal-fault rule removed
        let modified = RuleSet::new(
            canonical_constraint_rules().rules
                .into_iter()
                .filter(|r| r.name != "universal-fault")
                .collect()
        );
        let check = pinned.check_candidate(&modified);
        assert!(!check.is_valid,
            "removing pinned rule must fail check");
        assert!(check.missing_rules.iter().any(|s| s == "universal-fault"));
    }

    // ── Synthesis cycle guard ─────────────────────────────────────────────

    #[test]
    fn canonical_spec_accepted_by_guard() {
        let mut guard = SynthesisCycleGuard::new();
        let rs = canonical_constraint_rules();
        let acceptance = guard.evaluate(&rs);
        assert!(acceptance.accepted,
            "canonical spec must be accepted by cycle guard: {:?}",
            acceptance.reasons_for_rejection());
    }

    #[test]
    fn empty_spec_rejected_by_guard() {
        let mut guard = SynthesisCycleGuard::new();
        let empty = RuleSet::new(vec![]);
        let acceptance = guard.evaluate(&empty);
        assert!(!acceptance.accepted,
            "empty spec must be rejected by cycle guard");
        assert!(!acceptance.hard_violations.is_empty());
    }

    #[test]
    fn guard_tracks_history() {
        let mut guard = SynthesisCycleGuard::new();
        let rs = canonical_constraint_rules();
        for _ in 0..3 { guard.evaluate(&rs); }
        assert_eq!(guard.cycle_count(), 3);
        assert_eq!(guard.acceptance_rate(), 1.0);
    }

    #[test]
    fn degraded_spec_eventually_rejected() {
        let mut guard = SynthesisCycleGuard::new();
        // First cycle: accept canonical (sets baseline)
        let rs = canonical_constraint_rules();
        let a1 = guard.evaluate(&rs);
        assert!(a1.accepted, "canonical must be accepted");

        // Second cycle: severely degraded (empty)
        let empty = RuleSet::new(vec![]);
        let a2 = guard.evaluate(&empty);
        assert!(!a2.accepted, "severely degraded spec must be rejected");
    }

    #[test]
    fn cycle_acceptance_captures_entropy_delta() {
        let mut guard = SynthesisCycleGuard::new();
        let rs = canonical_constraint_rules();
        guard.evaluate(&rs); // baseline
        let acceptance = guard.evaluate(&rs); // same spec
        assert!(acceptance.entropy_delta.is_some(),
            "second cycle must have entropy delta");
        let delta = acceptance.entropy_delta.unwrap();
        assert_eq!(delta.expressiveness_delta, 0.0,
            "same spec → zero expressiveness delta");
    }

    // ── Invariant report ──────────────────────────────────────────────────

    #[test]
    fn invariant_report_classifies_correctly() {
        let rs = canonical_constraint_rules();
        let report = InvariantCheckReport::run(&rs, &standard_invariants());
        assert_eq!(report.hard_violations, 0);
        assert!(report.is_acceptable);
        println!("Invariant report: {} hard, {} soft violations",
            report.hard_violations, report.soft_violations);
    }
}
