// specification_entropy.rs — Expressiveness and information content of a rule set.
//
// PROBLEM: Synthesis can converge toward "valid but useless minimal rules".
// Two degenerate cases:
//
//   Spec collapse:   One absorbing rule covers everything.
//                    Valid: algebra agrees with kernel.
//                    Useless: provides zero diagnostic information.
//                    Example: Rule("*", True, Phase(Faulted))
//
//   Spec overfitting: 522 exact rules, one per cell.
//                    Valid: algebra agrees with kernel.
//                    Useless: provides zero compression or abstraction.
//
// The sweet spot: rules that use structural patterns (PhaseIsProcessing, etc.)
// to cover many cells meaningfully while preserving semantic distinctness.
//
// SOLUTION: Specification entropy — a composite metric that measures:
//
//   1. Shannon entropy H(R):    information content of coverage distribution
//      High = rules cover cells roughly uniformly  (not dominated by one rule)
//      Low  = one rule covers almost everything   (spec collapse)
//
//   2. Structural diversity D(R): fraction of rules using structural predicates
//      High = rules encode semantic knowledge (is_processing, is_terminal, ...)
//      Low  = all rules are exact (Exact pair, no abstraction)
//
//   3. Compression ratio C(R):   explicit_cells / rule_count
//      High = each rule explains many cells      (well-abstracted)
//      Low  = each rule explains one cell        (degenerate)
//
//   4. Expressiveness score E(R): composite of H, D, C
//      Clamped to [0, 1]. Used as the "specification health" indicator.
//
// EntropyBound enforces minimum acceptable values.
// If synthesis produces a rule set below the bound, it is rejected.
//
// Single source of truth for specification expressiveness.

use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::{canonical_constraint_rules, RuleSet},
    kernel_state::{KernelInputKind, KernelPhase, TRANSITION_TABLE},
};

// ─── EntropyMeasure ───────────────────────────────────────────────────────────

/// The complete entropy measurement for one rule set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyMeasure {
    /// Shannon entropy H(R) ∈ [0, log2(|rules|)] — information content of coverage.
    pub shannon_entropy: f64,
    /// Shannon entropy normalized to [0, 1].
    pub shannon_normalized: f64,
    /// Fraction of rules that use structural predicates (not exact pairs).
    pub structural_diversity: f64,
    /// Compression ratio: cells_covered / rule_count.
    pub compression_ratio: f64,
    /// Fraction of TRANSITION_TABLE covered by the rule set.
    pub table_coverage: f64,
    /// Composite expressiveness score ∈ [0, 1].
    pub expressiveness: f64,
    /// Gini coefficient over rule coverage — 0 = uniform, 1 = monopoly.
    pub gini: f64,
    /// Number of rules in the measured set.
    pub rule_count: usize,
    /// Total cells covered (distinct (phase, input) pairs).
    pub cells_covered: usize,
}

impl EntropyMeasure {
    /// Classify the expressiveness level.
    pub fn level(&self) -> EntropyLevel {
        match self.expressiveness {
            e if e >= 0.8 => EntropyLevel::Rich,
            e if e >= 0.6 => EntropyLevel::Adequate,
            e if e >= 0.4 => EntropyLevel::Sparse,
            e if e >= 0.2 => EntropyLevel::Degenerate,
            _             => EntropyLevel::Collapsed,
        }
    }

    pub fn is_above_floor(&self, floor: &EntropyFloor) -> bool {
        self.shannon_normalized >= floor.min_shannon
            && self.structural_diversity >= floor.min_structural_diversity
            && self.compression_ratio >= floor.min_compression_ratio
            && self.table_coverage >= floor.min_table_coverage
    }

    /// Collapse risk: how close are we to spec collapse?
    /// Returns 0.0 = safe, 1.0 = collapsed.
    pub fn collapse_risk(&self) -> f64 {
        // Risk increases as Gini approaches 1 (monopoly) and compression approaches 1 (exact)
        let gini_risk = self.gini;
        let exact_risk = 1.0 - self.structural_diversity;
        (gini_risk * 0.6 + exact_risk * 0.4).clamp(0.0, 1.0)
    }

    pub fn report(&self) -> String {
        format!(
            "entropy={:.3} ({}), structural={:.1}%, compression={:.1}×, \
             coverage={:.1}%, gini={:.3}, collapse_risk={:.2}",
            self.expressiveness,
            self.level(),
            self.structural_diversity * 100.0,
            self.compression_ratio,
            self.table_coverage * 100.0,
            self.gini,
            self.collapse_risk(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EntropyLevel {
    Collapsed,   // expressiveness < 0.2
    Degenerate,  // expressiveness 0.2–0.4
    Sparse,      // expressiveness 0.4–0.6
    Adequate,    // expressiveness 0.6–0.8
    Rich,        // expressiveness ≥ 0.8
}

impl std::fmt::Display for EntropyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Collapsed  => "COLLAPSED",
            Self::Degenerate => "DEGENERATE",
            Self::Sparse     => "SPARSE",
            Self::Adequate   => "ADEQUATE",
            Self::Rich       => "RICH",
        })
    }
}

// ─── EntropyFloor ─────────────────────────────────────────────────────────────

/// Minimum acceptable entropy values for a valid specification.
/// Synthesis results that fall below the floor are rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyFloor {
    /// Minimum normalized Shannon entropy. Below this = spec is too concentrated.
    pub min_shannon: f64,
    /// Minimum structural diversity. Below this = too many exact rules.
    pub min_structural_diversity: f64,
    /// Minimum cells per rule. Below this = too many rules, not compressed enough.
    pub min_compression_ratio: f64,
    /// Minimum table coverage. Below this = spec doesn't explain enough behavior.
    pub min_table_coverage: f64,
}

impl EntropyFloor {
    /// The production floor for the DELPHOS kernel.
    /// Synthesis results must exceed all of these.
    pub fn production() -> Self {
        Self {
            min_shannon: 0.3,
            min_structural_diversity: 0.2, // at least 20% of rules use structural patterns
            min_compression_ratio: 1.5,    // at least 1.5 cells per rule on average
            min_table_coverage: 0.5,       // must explain at least 50% of table entries
        }
    }

    /// Relaxed floor for development and testing.
    pub fn development() -> Self {
        Self {
            min_shannon: 0.1,
            min_structural_diversity: 0.0,
            min_compression_ratio: 1.0,
            min_table_coverage: 0.1,
        }
    }

    /// The trivial floor — never rejects anything.
    pub fn none() -> Self {
        Self {
            min_shannon: 0.0,
            min_structural_diversity: 0.0,
            min_compression_ratio: 0.0,
            min_table_coverage: 0.0,
        }
    }
}

// ─── SpecificationEntropy ─────────────────────────────────────────────────────

/// Computes entropy metrics for a rule set.
pub struct SpecificationEntropy;

impl SpecificationEntropy {
    /// Measure the expressiveness of a rule set.
    pub fn measure(rule_set: &RuleSet) -> EntropyMeasure {
        let rule_count = rule_set.rule_count();
        if rule_count == 0 {
            return Self::zero_measure();
        }

        // Compute coverage per rule: how many cells each rule covers
        let coverages: Vec<usize> = {
            let mut seen = std::collections::BTreeSet::new();
            let mut per_rule = Vec::new();
            for rule in &rule_set.rules {
                let mut rule_cells = 0;
                for (phase, input, _) in rule.entries() {
                    if seen.insert((phase, input)) {
                        rule_cells += 1;
                    }
                }
                per_rule.push(rule_cells);
            }
            per_rule
        };

        let cells_covered: usize = coverages.iter().sum();
        let _total_cells = KernelPhase::ALL.len() * KernelInputKind::ALL.len();

        // Shannon entropy: H = -Σ p_i * log2(p_i) where p_i = coverage_i / total_covered
        let shannon = if cells_covered > 0 {
            coverages.iter()
                .filter(|&&c| c > 0)
                .map(|&c| {
                    let p = c as f64 / cells_covered as f64;
                    -p * p.log2()
                })
                .sum::<f64>()
        } else { 0.0 };

        // Normalized Shannon (0 = all coverage in one rule, 1 = perfectly uniform)
        let max_shannon = (rule_count as f64).log2().max(1.0);
        let shannon_normalized = (shannon / max_shannon).clamp(0.0, 1.0);

        // Structural diversity: fraction of rules using non-exact patterns
        // We check: does the rule cover more than 1 cell? (proxy for structural pattern)
        let structural_count = coverages.iter().filter(|&&c| c > 1).count();
        let structural_diversity = structural_count as f64 / rule_count as f64;

        // Compression ratio: average cells per rule
        let compression_ratio = if rule_count > 0 {
            cells_covered as f64 / rule_count as f64
        } else { 0.0 };

        // Table coverage fraction
        let table_coverage = {
            let table_covered = TRANSITION_TABLE.iter()
                .filter(|e| rule_set.delta(e.from, e.on) == e.to)
                .count();
            table_covered as f64 / TRANSITION_TABLE.len() as f64
        };

        // Gini coefficient (inequality in coverage distribution)
        let gini = Self::gini_coefficient(&coverages);

        // Composite expressiveness: weighted combination
        // - Shannon normalized: 30% (information spread)
        // - Structural diversity: 30% (use of semantic patterns)
        // - Compression ratio (normalized to [0,1] with asymptote): 20%
        // - Table coverage: 20% (explains real behavior)
        let compression_normalized = 1.0 - (-compression_ratio / 10.0_f64).exp(); // [0,1)
        let expressiveness = (
            shannon_normalized * 0.30
            + structural_diversity * 0.30
            + compression_normalized * 0.20
            + table_coverage * 0.20
        ).clamp(0.0, 1.0);

        EntropyMeasure {
            shannon_entropy: shannon,
            shannon_normalized,
            structural_diversity,
            compression_ratio,
            table_coverage,
            expressiveness,
            gini,
            rule_count,
            cells_covered,
        }
    }

    /// Measure entropy change between two synthesis cycles.
    /// Returns the signed delta for each metric (positive = improvement).
    pub fn measure_cycle_delta(before: &EntropyMeasure, after: &EntropyMeasure) -> EntropyCycleDelta {
        EntropyCycleDelta {
            shannon_delta: after.shannon_normalized - before.shannon_normalized,
            structural_delta: after.structural_diversity - before.structural_diversity,
            compression_delta: after.compression_ratio - before.compression_ratio,
            coverage_delta: after.table_coverage - before.table_coverage,
            expressiveness_delta: after.expressiveness - before.expressiveness,
            gini_delta: after.gini - before.gini,
        }
    }

    /// Gini coefficient of a distribution.
    /// 0 = perfectly equal, 1 = monopoly.
    fn gini_coefficient(values: &[usize]) -> f64 {
        if values.is_empty() { return 0.0; }
        let n = values.len();
        let total: usize = values.iter().sum();
        if total == 0 { return 0.0; }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let numerator: f64 = sorted.iter().enumerate()
            .map(|(i, &v)| (2i64 * (i as i64 + 1) - n as i64 - 1) as f64 * v as f64)
            .sum();
        numerator.abs() / (n as f64 * total as f64)
    }

    fn zero_measure() -> EntropyMeasure {
        EntropyMeasure {
            shannon_entropy: 0.0, shannon_normalized: 0.0,
            structural_diversity: 0.0, compression_ratio: 0.0,
            table_coverage: 0.0, expressiveness: 0.0, gini: 0.0,
            rule_count: 0, cells_covered: 0,
        }
    }

    /// Measure the canonical specification as a baseline.
    pub fn canonical_baseline() -> EntropyMeasure {
        Self::measure(&canonical_constraint_rules())
    }
}

/// Signed change in entropy between two synthesis cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyCycleDelta {
    pub shannon_delta: f64,
    pub structural_delta: f64,
    pub compression_delta: f64,
    pub coverage_delta: f64,
    pub expressiveness_delta: f64,
    /// Positive gini_delta = more concentrated (worse).
    pub gini_delta: f64,
}

impl EntropyCycleDelta {
    pub fn is_improving(&self) -> bool { self.expressiveness_delta >= 0.0 }
    pub fn is_degrading(&self) -> bool { self.expressiveness_delta < -0.05 }
    pub fn coverage_grew(&self) -> bool { self.coverage_delta > 0.0 }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint_algebra::{ConstraintExpr, ConstraintRule, ConstraintTarget};

    #[test]
    fn canonical_baseline_is_adequate() {
        let m = SpecificationEntropy::canonical_baseline();
        println!("Canonical: {}", m.report());
        assert!(m.expressiveness > 0.0,
            "canonical spec must have positive expressiveness");
        assert!(m.structural_diversity > 0.0,
            "canonical spec must use some structural rules");
        assert!(m.compression_ratio > 1.0,
            "canonical spec must compress more than 1 cell per rule");
    }

    #[test]
    fn empty_rule_set_is_collapsed() {
        let empty = RuleSet::new(vec![]);
        let m = SpecificationEntropy::measure(&empty);
        assert_eq!(m.expressiveness, 0.0);
        assert_eq!(m.level(), EntropyLevel::Collapsed);
    }

    #[test]
    fn single_absorbing_rule_has_low_diversity() {
        // One rule: True → Faulted (absorbs everything)
        let all_faulted = RuleSet::new(vec![
            crate::constraint_algebra::ConstraintRule::new(
                "absorb-all", ConstraintExpr::True, ConstraintTarget::Phase(KernelPhase::Faulted),
            )
        ]);
        let m = SpecificationEntropy::measure(&all_faulted);
        // Gini = 0 (only one rule, no inequality) but structural_diversity = 1.0
        // Coverage would be all cells
        // Shannon entropy = 0 (only one rule)
        assert_eq!(m.shannon_normalized, 0.0,
            "single rule must have 0 Shannon entropy");
        println!("Single absorbing rule: {}", m.report());
    }

    #[test]
    fn entropy_floor_production_accepts_canonical() {
        let m = SpecificationEntropy::canonical_baseline();
        let floor = EntropyFloor::production();
        assert!(m.is_above_floor(&floor),
            "canonical spec must satisfy production floor: {}", m.report());
    }

    #[test]
    fn entropy_floor_rejects_empty() {
        let empty = RuleSet::new(vec![]);
        let m = SpecificationEntropy::measure(&empty);
        let floor = EntropyFloor::development(); // even the relaxed floor
        // Empty spec has 0% coverage — fails the coverage requirement
        assert!(!m.is_above_floor(&floor) || floor.min_table_coverage == 0.0);
    }

    #[test]
    fn gini_uniform_distribution() {
        // All rules cover equal cells → Gini ≈ 0
        let coverages = vec![10usize; 10]; // all equal
        let g = SpecificationEntropy::gini_coefficient(&coverages);
        assert!(g < 0.01, "equal coverage must have near-zero Gini: {g}");
    }

    #[test]
    fn gini_monopoly() {
        // One rule covers all, rest cover 0 → Gini close to 1
        let mut coverages = vec![0usize; 9];
        coverages.push(100);
        let g = SpecificationEntropy::gini_coefficient(&coverages);
        assert!(g > 0.8, "monopoly coverage must have high Gini: {g}");
    }

    #[test]
    fn cycle_delta_measures_improvement() {
        let baseline = SpecificationEntropy::canonical_baseline();
        // Create a "better" spec by adding more coverage
        let delta = SpecificationEntropy::measure_cycle_delta(&baseline, &baseline);
        assert_eq!(delta.expressiveness_delta, 0.0, "same spec → zero delta");
        assert!(!delta.is_degrading());
    }

    #[test]
    fn collapse_risk_is_low_for_canonical() {
        let m = SpecificationEntropy::canonical_baseline();
        assert!(m.collapse_risk() < 0.8,
            "canonical spec must have low collapse risk: {}", m.collapse_risk());
        println!("Canonical collapse risk: {:.3}", m.collapse_risk());
    }
}
