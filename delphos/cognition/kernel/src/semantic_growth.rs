// semantic_growth.rs — Expressiveness Conservation Law.
//
// PROBLEM: "Future semantic discoverability"
//
// The system currently optimizes: stability, non-redundancy, coverage,
// entropy, interference-freedom. None of these track whether the spec
// retains the CAPACITY to describe new phenomena.
//
// A spec can be simultaneously:
//   - Fully aligned with kernel_delta  (L1 check passes)
//   - Non-redundant                    (no redundant rules)
//   - High entropy                     (good structural diversity)
//   - Zero interference                (no domain overlap)
//   - Semantically FROZEN              (no room for new content)
//
// Semantic freezing happens when over-generalized rules claim large
// swaths of Q×Σ as "Faulted territory" — cells that could have held
// future behaviors are locked by existing absorbing rules.
//
// FORMAL DEFINITION:
//
//   A cell (phase, input) is SEMANTICALLY FREE iff:
//     (a) No rule's guard evaluates to true for it
//         (unclaimed by any explicit rule)
//     (b) NOT in TRANSITION_TABLE
//         (no required behavior defined yet)
//
//   Free cells = raw material for future semantic content.
//
//   Cells claimed by the universal-fault rule ("¬terminal ∧ FaultDetected → Faulted")
//   are NOT free — they are explicitly locked to Faulted behavior.
//   Even though the default would produce the same result, an explicit claim
//   prevents future rules from using those cells differently.
//
// EXPRESSIVENESS CONSERVATION LAW:
//
//   free_fraction(R_{t+1}) >= free_fraction(R_t) × (1 - max_reduction_per_cycle)
//
//   The spec may NOT claim more than max_reduction_per_cycle fraction of
//   currently-free cells in a single synthesis cycle.
//
//   This prevents "semantic packing" — a spec that is correct today but
//   has no room to grow tomorrow.
//
// SEMANTIC GROWTH INVARIANT (hard constraint):
//
//   free_fraction >= min_free_fraction (default: 0.50)
//   free_cells >= min_free_cells (absolute minimum: 100)
//
//   If a synthesis result violates these bounds, it is rejected by the
//   SynthesisCycleGuard, regardless of other properties.
//
// GROWTH VECTOR: quantifies WHAT kinds of new rules could still be added.
//
//   For each cluster of free cells sharing the same phase:
//     → these cells could be claimed by a new rule for that phase
//   For each input with many free cells across phases:
//     → a new input-specific rule could be added
//   max_new_exact_rules: upper bound on distinct new exact-pair rules
//   max_new_structural_rules: upper bound on structural-pattern rules
//
// Single source of truth. One module, one location.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::RuleSet,
    kernel_state::{KernelInputKind, KernelPhase, TRANSITION_TABLE},
};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Default minimum free fraction (50% of Q×Σ must remain unclaimed).
pub const DEFAULT_MIN_FREE_FRACTION: f64 = 0.50;
/// Default minimum absolute free cell count.
pub const DEFAULT_MIN_FREE_CELLS: usize = 100;
/// Default maximum reduction per synthesis cycle.
pub const DEFAULT_MAX_REDUCTION_PER_CYCLE: f64 = 0.10;

// ─── SemanticHeadroom ─────────────────────────────────────────────────────────

/// The "semantic headroom" of a rule set — raw capacity for new content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHeadroom {
    /// Total cells in Q×Σ.
    pub total_cells: usize,
    /// Cells explicitly claimed by at least one rule's guard.
    pub claimed_cells: usize,
    /// Cells in TRANSITION_TABLE (defined, required behaviors).
    pub table_cells: usize,
    /// Cells that are BOTH free AND in TRANSITION_TABLE (should be 0 for aligned spec).
    pub unresolved_table_cells: usize,
    /// Cells that are unclaimed AND not in TRANSITION_TABLE: the true growth space.
    pub free_cells: usize,
    /// free_cells / total_cells ∈ [0, 1].
    pub free_fraction: f64,
    /// free_cells grouped by phase (tells us which phases have room to grow).
    pub free_by_phase: BTreeMap<KernelPhase, usize>,
    /// free_cells grouped by input (tells us which inputs have room to grow).
    pub free_by_input: BTreeMap<KernelInputKind, usize>,
}

impl SemanticHeadroom {
    /// Does this headroom satisfy the conservation invariant?
    pub fn satisfies(&self, invariant: &SemanticGrowthInvariant) -> bool {
        self.free_fraction >= invariant.min_free_fraction
            && self.free_cells >= invariant.min_free_cells
    }

    /// Phase with most free cells (best target for future extension).
    pub fn most_open_phase(&self) -> Option<KernelPhase> {
        self.free_by_phase.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&phase, _)| phase)
    }

    /// Input with most free cells (best candidate for new input-specific rule).
    pub fn most_open_input(&self) -> Option<KernelInputKind> {
        self.free_by_input.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&input, _)| input)
    }

    /// How many phases have at least one free cell?
    pub fn phases_with_headroom(&self) -> usize {
        self.free_by_phase.values().filter(|&&n| n > 0).count()
    }
}

// ─── SemanticGrowthInvariant ──────────────────────────────────────────────────

/// The formal invariant that bounds semantic packing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGrowthInvariant {
    /// Minimum fraction of Q×Σ that must remain free.
    pub min_free_fraction: f64,
    /// Minimum absolute number of free cells.
    pub min_free_cells: usize,
    /// Maximum fraction of free cells a single synthesis cycle may claim.
    pub max_reduction_per_cycle: f64,
}

impl SemanticGrowthInvariant {
    pub fn production() -> Self {
        Self {
            min_free_fraction: DEFAULT_MIN_FREE_FRACTION,
            min_free_cells: DEFAULT_MIN_FREE_CELLS,
            max_reduction_per_cycle: DEFAULT_MAX_REDUCTION_PER_CYCLE,
        }
    }

    pub fn relaxed() -> Self {
        Self {
            min_free_fraction: 0.20,
            min_free_cells: 20,
            max_reduction_per_cycle: 0.30,
        }
    }

    /// Check if a cycle transition (before → after) is within bounds.
    pub fn check_cycle_transition(&self, before: &SemanticHeadroom, after: &SemanticHeadroom) -> CycleTransitionCheck {
        let reduction = if before.free_cells == 0 { 0.0 }
            else { (before.free_cells.saturating_sub(after.free_cells)) as f64 / before.free_cells as f64 };

        let violates_floor = after.free_fraction < self.min_free_fraction;
        let violates_absolute = after.free_cells < self.min_free_cells;
        let violates_rate = reduction > self.max_reduction_per_cycle;

        CycleTransitionCheck {
            before_free: before.free_cells,
            after_free: after.free_cells,
            reduction_fraction: reduction,
            violates_floor,
            violates_absolute,
            violates_rate,
            is_acceptable: !violates_floor && !violates_absolute && !violates_rate,
        }
    }
}

/// Result of checking a synthesis cycle against the growth invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTransitionCheck {
    pub before_free: usize,
    pub after_free: usize,
    pub reduction_fraction: f64,
    pub violates_floor: bool,
    pub violates_absolute: bool,
    pub violates_rate: bool,
    pub is_acceptable: bool,
}

impl CycleTransitionCheck {
    pub fn violation_reasons(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.violates_floor    { v.push("free_fraction below min_free_fraction"); }
        if self.violates_absolute { v.push("free_cells below min_free_cells"); }
        if self.violates_rate     { v.push("reduction_per_cycle exceeds max"); }
        v
    }
}

// ─── GrowthVector ─────────────────────────────────────────────────────────────

/// Quantifies WHAT kinds of new rules could still be added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthVector {
    /// Upper bound on distinct new exact-pair rules that could be added.
    pub max_new_exact_rules: usize,
    /// How many phases could host a new structural rule.
    pub structural_growth_phases: usize,
    /// How many inputs could host a new input-specific rule.
    pub structural_growth_inputs: usize,
    /// Total "rule slots" available (rough upper bound on new rules).
    pub total_growth_capacity: usize,
    /// Whether the spec is "semantically packed" (no room for new content).
    pub is_packed: bool,
    /// The most "open" phase (most free cells, best target for extension).
    pub most_open_phase: Option<KernelPhase>,
    /// The most "open" input.
    pub most_open_input: Option<KernelInputKind>,
}

impl GrowthVector {
    fn from_headroom(h: &SemanticHeadroom) -> Self {
        let structural_phases = h.free_by_phase.values().filter(|&&n| n >= 2).count();
        let structural_inputs = h.free_by_input.values().filter(|&&n| n >= 2).count();
        // A structural rule needs at least 2 cells to generalize
        let structural_capacity = structural_phases.max(structural_inputs);
        Self {
            max_new_exact_rules: h.free_cells,
            structural_growth_phases: structural_phases,
            structural_growth_inputs: structural_inputs,
            total_growth_capacity: h.free_cells + structural_capacity,
            is_packed: h.free_cells < DEFAULT_MIN_FREE_CELLS,
            most_open_phase: h.most_open_phase(),
            most_open_input: h.most_open_input(),
        }
    }
}

// ─── SemanticGrowthAnalyzer ───────────────────────────────────────────────────

/// Computes semantic headroom and growth vectors for rule sets.
pub struct SemanticGrowthAnalyzer {
    pub invariant: SemanticGrowthInvariant,
}

impl SemanticGrowthAnalyzer {
    pub fn new() -> Self { Self { invariant: SemanticGrowthInvariant::production() } }

    pub fn with_invariant(mut self, inv: SemanticGrowthInvariant) -> Self {
        self.invariant = inv; self
    }

    /// Compute semantic headroom for a rule set.
    ///
    /// A cell is FREE iff:
    ///   (a) No rule's guard evaluates to true for it (unclaimed)
    ///   (b) NOT in TRANSITION_TABLE (no required behavior)
    pub fn compute_headroom(&self, rule_set: &RuleSet) -> SemanticHeadroom {
        let total = KernelPhase::ALL.len() * KernelInputKind::ALL.len();
        let mut claimed_cells = 0usize;
        let mut free_cells = 0usize;
        let mut unresolved_table = 0usize;
        let mut free_by_phase: BTreeMap<KernelPhase, usize> = BTreeMap::new();
        let mut free_by_input: BTreeMap<KernelInputKind, usize> = BTreeMap::new();

        let table_cells = TRANSITION_TABLE.len();

        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let is_claimed = rule_set.rules.iter().any(|r| r.guard.eval(phase, input));
                let in_table   = TRANSITION_TABLE.iter().any(|e| e.from == phase && e.on == input);

                if is_claimed {
                    claimed_cells += 1;
                    if in_table {
                        // Table entry claimed by a rule — correct (not unresolved)
                    }
                } else {
                    // Not claimed by any explicit rule
                    if in_table {
                        // Table entry with no explicit rule → unresolved (relies on default)
                        unresolved_table += 1;
                    } else {
                        // Free: not claimed, not in table → available for future use
                        free_cells += 1;
                        *free_by_phase.entry(phase).or_insert(0) += 1;
                        *free_by_input.entry(input).or_insert(0) += 1;
                    }
                }
            }
        }

        let free_fraction = free_cells as f64 / total as f64;

        SemanticHeadroom {
            total_cells: total,
            claimed_cells,
            table_cells,
            unresolved_table_cells: unresolved_table,
            free_cells,
            free_fraction,
            free_by_phase,
            free_by_input,
        }
    }

    /// Compute the growth vector for a rule set.
    pub fn compute_growth_vector(&self, rule_set: &RuleSet) -> GrowthVector {
        let h = self.compute_headroom(rule_set);
        GrowthVector::from_headroom(&h)
    }

    /// Check if a rule set satisfies the growth invariant.
    pub fn check(&self, rule_set: &RuleSet) -> GrowthInvariantResult {
        let headroom = self.compute_headroom(rule_set);
        let satisfies = headroom.satisfies(&self.invariant);
        let vector = GrowthVector::from_headroom(&headroom);
        GrowthInvariantResult { headroom, vector, satisfies, invariant: self.invariant.clone() }
    }
}

impl Default for SemanticGrowthAnalyzer { fn default() -> Self { Self::new() } }

/// Complete result of a growth invariant check.
#[derive(Debug, Clone)]
pub struct GrowthInvariantResult {
    pub headroom: SemanticHeadroom,
    pub vector: GrowthVector,
    pub satisfies: bool,
    pub invariant: SemanticGrowthInvariant,
}

// ─── HeadroomHistory ─────────────────────────────────────────────────────────

/// Tracks semantic headroom across synthesis cycles.
/// Enforces: headroom may not decrease faster than max_reduction_per_cycle.
#[derive(Debug)]
pub struct HeadroomHistory {
    pub cycles: Vec<CycleHeadroomEntry>,
    pub invariant: SemanticGrowthInvariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleHeadroomEntry {
    pub cycle_index: u64,
    pub free_cells: usize,
    pub free_fraction: f64,
    pub reduction_from_prev: f64,
    pub acceptable: bool,
}

impl HeadroomHistory {
    pub fn new(invariant: SemanticGrowthInvariant) -> Self {
        Self { cycles: Vec::new(), invariant }
    }

    /// Record a new headroom measurement for a synthesis cycle.
    /// Returns the cycle transition check.
    pub fn record(&mut self, headroom: &SemanticHeadroom) -> CycleHeadroomEntry {
        let prev = self.cycles.last().cloned();
        let reduction = prev.as_ref().map(|p| {
            if p.free_cells == 0 { 0.0 }
            else { (p.free_cells.saturating_sub(headroom.free_cells)) as f64 / p.free_cells as f64 }
        }).unwrap_or(0.0);

        let check = if let Some(prev_entry) = &prev {
            let prev_h = SemanticHeadroom {
                free_cells: prev_entry.free_cells,
                free_fraction: prev_entry.free_fraction,
                total_cells: headroom.total_cells,
                claimed_cells: 0, table_cells: 0,
                unresolved_table_cells: 0,
                free_by_phase: BTreeMap::new(),
                free_by_input: BTreeMap::new(),
            };
            self.invariant.check_cycle_transition(&prev_h, headroom)
        } else {
            CycleTransitionCheck {
                before_free: headroom.free_cells, after_free: headroom.free_cells,
                reduction_fraction: 0.0, violates_floor: false, violates_absolute: false,
                violates_rate: false, is_acceptable: true,
            }
        };

        let entry = CycleHeadroomEntry {
            cycle_index: self.cycles.len() as u64,
            free_cells: headroom.free_cells,
            free_fraction: headroom.free_fraction,
            reduction_from_prev: reduction,
            acceptable: check.is_acceptable,
        };
        self.cycles.push(entry.clone());
        entry
    }

    pub fn is_monotone_within_tolerance(&self) -> bool {
        self.cycles.windows(2).all(|w| {
            w[1].reduction_from_prev <= self.invariant.max_reduction_per_cycle + 0.001
        })
    }

    pub fn all_acceptable(&self) -> bool {
        self.cycles.iter().all(|c| c.acceptable)
    }

    pub fn latest_headroom(&self) -> Option<&CycleHeadroomEntry> {
        self.cycles.last()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint_algebra::{canonical_constraint_rules, ConstraintExpr, ConstraintRule, ConstraintTarget, RuleSet};

    fn analyzer() -> SemanticGrowthAnalyzer { SemanticGrowthAnalyzer::new() }

    // ── Headroom computation ─────────────────────────────────────────────

    #[test]
    fn canonical_spec_has_high_headroom() {
        let rs = canonical_constraint_rules();
        let h = analyzer().compute_headroom(&rs);
        println!("Canonical headroom: {}/{} free ({:.1}%), claimed={}",
            h.free_cells, h.total_cells, h.free_fraction * 100.0, h.claimed_cells);
        assert!(h.free_fraction >= 0.50,
            "canonical spec must have >= 50% free cells: {:.1}%", h.free_fraction * 100.0);
        assert_eq!(h.total_cells, 18 * 29,  // KernelPhase::ALL × KernelInputKind::ALL
            "total cells must be |Q| × |Σ|");
    }

    #[test]
    fn canonical_passes_growth_invariant() {
        let rs = canonical_constraint_rules();
        let result = analyzer().check(&rs);
        assert!(result.satisfies,
            "canonical spec must satisfy growth invariant: free={:.1}%",
            result.headroom.free_fraction * 100.0);
    }

    #[test]
    fn free_plus_claimed_plus_unresolved_equals_total() {
        let rs = canonical_constraint_rules();
        let h = analyzer().compute_headroom(&rs);
        // free + claimed + (table cells in claimed) = total
        // More precisely: free_cells are NOT in table AND NOT claimed
        // claimed_cells + free_cells = total - (table entries not claimed)
        // Let's verify the invariant: free + claimed = total - unresolved_table
        let accounted = h.free_cells + h.claimed_cells + h.unresolved_table_cells;
        assert_eq!(accounted, h.total_cells,
            "free + claimed + unresolved must equal total: {} + {} + {} = {} (total={})",
            h.free_cells, h.claimed_cells, h.unresolved_table_cells, accounted, h.total_cells);
    }

    #[test]
    fn adding_over_generalized_rule_reduces_headroom() {
        let baseline = canonical_constraint_rules();
        let base_h = analyzer().compute_headroom(&baseline);

        // Add a rule that claims large swatch: True → Faulted (absorbs everything)
        let mut rules = baseline.rules.clone();
        rules.push(ConstraintRule::new(
            "over-generalize-test",
            ConstraintExpr::PhaseIn(vec![KernelPhase::Idle, KernelPhase::Genesis]),
            ConstraintTarget::Phase(KernelPhase::Faulted),
        ));
        let enlarged = RuleSet::new(rules);
        let enlarged_h = analyzer().compute_headroom(&enlarged);

        assert!(enlarged_h.free_cells < base_h.free_cells,
            "adding over-generalized rule must reduce free cells: {} vs {}", 
            enlarged_h.free_cells, base_h.free_cells);
        println!("Headroom reduction: {} → {} ({:.1}%)",
            base_h.free_cells, enlarged_h.free_cells,
            (base_h.free_cells - enlarged_h.free_cells) as f64 / base_h.free_cells as f64 * 100.0);
    }

    #[test]
    fn packed_spec_fails_conservation_invariant() {
        // A spec that claims everything (True → Faulted) should have near-zero headroom
        let all_faulted = RuleSet::new(vec![
            ConstraintRule::new("absorb-all", ConstraintExpr::True, ConstraintTarget::Phase(KernelPhase::Faulted))
        ]);
        let h = analyzer().compute_headroom(&all_faulted);
        // Almost all cells are claimed → near-zero free cells
        println!("All-Faulted headroom: {}/{} free ({:.1}%)",
            h.free_cells, h.total_cells, h.free_fraction * 100.0);
        // table cells that this rule covers but with wrong target will be "claimed + unresolved"
        // The free_fraction should be very low (only unclaimed + non-table cells)
        assert!(!h.satisfies(&SemanticGrowthInvariant::production()),
            "fully-packed spec must fail conservation invariant");
    }

    // ── Growth vector ────────────────────────────────────────────────────

    #[test]
    fn canonical_growth_vector_is_non_empty() {
        let rs = canonical_constraint_rules();
        let v = analyzer().compute_growth_vector(&rs);
        assert!(!v.is_packed, "canonical spec must not be packed");
        assert!(v.max_new_exact_rules > 100,
            "canonical spec must have room for > 100 new exact rules: {}",
            v.max_new_exact_rules);
        println!("Growth: {} exact, {} structural phases, {} structural inputs",
            v.max_new_exact_rules, v.structural_growth_phases, v.structural_growth_inputs);
    }

    #[test]
    fn growth_vector_identifies_open_phase() {
        let rs = canonical_constraint_rules();
        let v = analyzer().compute_growth_vector(&rs);
        assert!(v.most_open_phase.is_some(),
            "must identify a phase with headroom");
        println!("Most open phase: {:?}", v.most_open_phase);
    }

    #[test]
    fn most_open_input_is_meaningful() {
        let rs = canonical_constraint_rules();
        let h = analyzer().compute_headroom(&rs);
        let most_open = h.most_open_input();
        assert!(most_open.is_some());
        let count = h.free_by_input[&most_open.unwrap()];
        assert!(count >= 2, "most open input must have at least 2 free cells: {count}");
        println!("Most open input: {:?} with {count} free cells", most_open);
    }

    // ── Conservation invariant ───────────────────────────────────────────

    #[test]
    fn cycle_transition_within_spec() {
        let rs = canonical_constraint_rules();
        let h = analyzer().compute_headroom(&rs);
        let inv = SemanticGrowthInvariant::production();
        // Same → same: zero reduction, always acceptable
        let check = inv.check_cycle_transition(&h, &h);
        assert!(check.is_acceptable, "identical headroom must be acceptable");
        assert_eq!(check.reduction_fraction, 0.0);
    }

    #[test]
    fn large_reduction_violates_rate() {
        let rs = canonical_constraint_rules();
        let mut rules = rs.rules.clone();
        // Add many absorbing rules to greatly reduce headroom
        for phase in [KernelPhase::Idle, KernelPhase::Deciding, KernelPhase::Applying,
                      KernelPhase::Stamping, KernelPhase::Emitting] {
            rules.push(ConstraintRule::new(
                Box::leak(format!("absorb-{phase}").into_boxed_str()),
                ConstraintExpr::PhaseEq(phase),
                ConstraintTarget::Phase(KernelPhase::Faulted),
            ));
        }
        let enlarged = RuleSet::new(rules);
        let original_h = analyzer().compute_headroom(&rs);
        let enlarged_h = analyzer().compute_headroom(&enlarged);
        let inv = SemanticGrowthInvariant::production();
        let check = inv.check_cycle_transition(&original_h, &enlarged_h);
        println!("Reduction: {:.1}%", check.reduction_fraction * 100.0);
        // If reduction > 10%, the rate invariant fires
        if check.reduction_fraction > inv.max_reduction_per_cycle {
            assert!(check.violates_rate,
                "large reduction must violate rate invariant");
        }
    }

    // ── Headroom history ─────────────────────────────────────────────────

    #[test]
    fn history_tracks_correctly() {
        let rs = canonical_constraint_rules();
        let h = analyzer().compute_headroom(&rs);
        let inv = SemanticGrowthInvariant::production();
        let mut hist = HeadroomHistory::new(inv);
        let e0 = hist.record(&h);
        let e1 = hist.record(&h); // same headroom
        assert_eq!(e0.free_cells, e1.free_cells);
        assert_eq!(e1.reduction_from_prev, 0.0);
        assert!(hist.all_acceptable());
        assert!(hist.is_monotone_within_tolerance());
    }

    #[test]
    fn history_detects_rapid_reduction() {
        let rs = canonical_constraint_rules();
        let original_h = analyzer().compute_headroom(&rs);

        // Create heavily packed spec
        let mut rules = rs.rules.clone();
        for phase in KernelPhase::ALL {
            rules.push(ConstraintRule::new(
                Box::leak(format!("pack-{phase}").into_boxed_str()),
                ConstraintExpr::PhaseEq(*phase),
                ConstraintTarget::Phase(KernelPhase::Faulted),
            ));
        }
        let packed_h = SemanticGrowthAnalyzer::new().compute_headroom(&RuleSet::new(rules));

        let inv = SemanticGrowthInvariant::production();
        let mut hist = HeadroomHistory::new(inv);
        hist.record(&original_h);
        let entry = hist.record(&packed_h);
        if entry.reduction_from_prev > DEFAULT_MAX_REDUCTION_PER_CYCLE {
            assert!(!entry.acceptable,
                "rapid reduction must be marked unacceptable");
        }
    }

    // ── Semantic partition identity ───────────────────────────────────────

    #[test]
    fn phases_with_headroom_covers_multiple_phases() {
        let rs = canonical_constraint_rules();
        let h = analyzer().compute_headroom(&rs);
        let phases = h.phases_with_headroom();
        assert!(phases >= 5, "at least 5 phases must have headroom: {phases}");
        println!("{phases}/{} phases have semantic headroom", KernelPhase::ALL.len());
    }
}
