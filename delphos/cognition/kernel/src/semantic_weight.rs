// semantic_weight.rs — Semantic relevance of individual rules.
//
// Three orthogonal scores that measure MEANING, not shape:
//
//   BehavioralNecessity — does removing this rule change a TRANSITION_TABLE output?
//     Critical:       rule is necessary for ≥1 table entry
//     Observational:  rule documents default or non-table behavior
//     Redundant:      another explicit rule already covers the same cells
//
//   CausalImportance — is this rule active for cells observed in execution traces?
//     Anti-gaming: sum of all rules' causal importance = 1.0 exactly.
//
//   StructuralSignificance — does the rule encode domain knowledge?
//     Grounded in predicate coverage (cell count). Not gameable without
//     changing rule semantics.
//
// Composite weight = 0.35×necessity + 0.35×causal + 0.20×structural + 0.10×reach
//
// Three gaming risks addressed:
//   Over-conservation: necessity=0 is the permission gate for simplification.
//   Over-compression:  causal importance detects unused generic rules.
//   Metric gaming:     all scores grounded in TRANSITION_TABLE + ExecutionTrace.
//
// Single source of truth for rule semantic relevance.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::{ConstraintRule, RuleSet},
    kernel_state::{KernelInputKind, KernelPhase, TRANSITION_TABLE},
    proof_certificate::ExecutionTrace,
};

// ─── NecessityClass ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NecessityClass {
    /// Rule necessary for ≥1 TRANSITION_TABLE entry. Cannot be removed.
    Critical,
    /// Rule documents default behavior or non-table cells. Has documentation value.
    Observational,
    /// Another explicit rule covers same cells with same target. Subsumed.
    Redundant,
}
impl std::fmt::Display for NecessityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Critical      => "CRITICAL",
            Self::Observational => "OBSERVATIONAL",
            Self::Redundant     => "REDUNDANT",
        })
    }
}

// ─── RuleNecessityProof ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleNecessityProof {
    pub rule_name: String,
    pub necessity_class: NecessityClass,
    pub critical_cells: Vec<(KernelPhase, KernelInputKind)>,
    pub observational_cells: Vec<(KernelPhase, KernelInputKind)>,
    pub redundant_cells: Vec<(KernelPhase, KernelInputKind)>,
    pub active_domain_size: usize,
    /// critical_cells / active_domain_size
    pub necessity_score: f64,
}
impl RuleNecessityProof {
    pub fn is_critical(&self)      -> bool { self.necessity_class == NecessityClass::Critical }
    pub fn is_observational(&self) -> bool { self.necessity_class == NecessityClass::Observational }
    pub fn is_redundant(&self)     -> bool { self.necessity_class == NecessityClass::Redundant }
}

// ─── RuleSemanticWeight ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSemanticWeight {
    pub rule_name: String,
    pub necessity: f64,
    pub necessity_class: NecessityClass,
    pub causal_importance: f64,
    pub structural_significance: f64,
    pub reachability: f64,
    pub semantic_weight: f64,
    pub recommendation: RuleRecommendation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleRecommendation {
    Preserve, Strengthen, Simplifiable, RemoveCandidate, DocumentationOnly,
}
impl std::fmt::Display for RuleRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Preserve         => "PRESERVE",
            Self::Strengthen       => "STRENGTHEN",
            Self::Simplifiable     => "SIMPLIFIABLE",
            Self::RemoveCandidate  => "REMOVE_CANDIDATE",
            Self::DocumentationOnly => "DOCUMENTATION_ONLY",
        })
    }
}

// ─── SemanticWeightLayer ─────────────────────────────────────────────────────

pub struct SemanticWeightLayer {
    trace_cells: BTreeMap<(KernelPhase, KernelInputKind), u64>,
    total_trace_observations: u64,
}

impl SemanticWeightLayer {
    pub fn new() -> Self { Self { trace_cells: BTreeMap::new(), total_trace_observations: 0 } }

    pub fn observe(&mut self, trace: &ExecutionTrace) {
        for cert in &trace.certificates {
            *self.trace_cells.entry((cert.from_phase, cert.input)).or_insert(0) += 1;
            self.total_trace_observations += 1;
        }
    }
    pub fn observe_all(&mut self, traces: &[ExecutionTrace]) { for t in traces { self.observe(t); } }
    pub fn observation_count(&self) -> u64 { self.total_trace_observations }

    // ── Necessity proof ─────────────────────────────────────────────────────

    /// Compute the formal necessity proof for rule at rule_index.
    ///
    /// A rule is Critical for cell (p,σ) if:
    ///   - it is the first (active) match, AND
    ///   - removing it changes the output, AND (p,σ) ∈ TRANSITION_TABLE
    ///
    /// A rule is Observational for cell (p,σ) if:
    /// - it is active, AND
    /// - fallback = default (no later explicit rule), AND output unchanged.
    ///   This case: rule documents implicit behavior — architectural intent.
    ///
    /// A rule is Redundant for cell (p,σ) if:
    /// - it is active, AND
    /// - another explicit later rule produces the same target
    pub fn necessity_proof(
        &self,
        rule: &ConstraintRule,
        rule_index: usize,
        all_rules: &[ConstraintRule],
    ) -> RuleNecessityProof {
        let mut critical_cells     = Vec::new();
        let mut observational_cells = Vec::new();
        let mut redundant_cells    = Vec::new();
        let mut active_domain      = 0usize;

        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if !rule.guard.eval(phase, input) { continue; }
                // Active: no earlier rule matched
                if all_rules[..rule_index].iter().any(|r| r.guard.eval(phase, input)) { continue; }
                active_domain += 1;
                let this_target = rule.target.resolve(phase);
                // What would the remaining rules produce?
                let later_match = all_rules[rule_index + 1..].iter()
                    .find(|r| r.guard.eval(phase, input));
                let fallback = later_match.map(|r| r.target.resolve(phase))
                    .unwrap_or(KernelPhase::Faulted);
                let fallback_from_explicit = later_match.is_some();
                let in_table = TRANSITION_TABLE.iter().any(|e| e.from == phase && e.on == input);

                if fallback != this_target {
                    // Removing changes output
                    if in_table { critical_cells.push((phase, input)); }
                    else        { observational_cells.push((phase, input)); }
                } else if fallback_from_explicit {
                    // Same output AND an explicit rule provides it → truly Redundant
                    redundant_cells.push((phase, input));
                } else {
                    // Same output AND fallback is the default — rule documents implicit behavior
                    observational_cells.push((phase, input));
                }
            }
        }

        let necessity_score = if active_domain == 0 { 0.0 } else {
            critical_cells.len() as f64 / active_domain as f64
        };
        let necessity_class = if !critical_cells.is_empty() {
            NecessityClass::Critical
        } else if !observational_cells.is_empty() {
            NecessityClass::Observational
        } else {
            NecessityClass::Redundant
        };

        RuleNecessityProof {
            rule_name: rule.name.to_string(), necessity_class,
            critical_cells, observational_cells, redundant_cells,
            active_domain_size: active_domain, necessity_score,
        }
    }

    // ── Causal importance ───────────────────────────────────────────────────

    pub fn causal_importance(&self, rule: &ConstraintRule, rule_index: usize, all_rules: &[ConstraintRule]) -> f64 {
        if self.total_trace_observations == 0 { return 0.0; }
        let active: u64 = self.trace_cells.iter()
            .filter(|&(&(phase, input), _)| {
                rule.guard.eval(phase, input)
                && !all_rules[..rule_index].iter().any(|r| r.guard.eval(phase, input))
            })
            .map(|(_, &count)| count)
            .sum();
        active as f64 / self.total_trace_observations as f64
    }

    // ── Structural significance ─────────────────────────────────────────────

    pub fn structural_significance(rule: &ConstraintRule) -> f64 {
        let cells = rule.guard.cardinality();
        let bonus = match cells { 0..=1 => 0.0, 2..=5 => 0.2, 6..=30 => 0.5, _ => 0.9 };
        let frac = cells as f64 / (KernelPhase::ALL.len() * KernelInputKind::ALL.len()) as f64;
        (frac * 0.5 + bonus * 0.5).clamp(0.05, 1.0)
    }

    // ── Reachability ─────────────────────────────────────────────────────────

    pub fn reachability(&self, rule: &ConstraintRule, rule_index: usize, all_rules: &[ConstraintRule]) -> f64 {
        let mut active = 0usize;
        let mut reached = 0usize;
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if !rule.guard.eval(phase, input) { continue; }
                if all_rules[..rule_index].iter().any(|r| r.guard.eval(phase, input)) { continue; }
                active += 1;
                if self.trace_cells.contains_key(&(phase, input)) { reached += 1; }
            }
        }
        if active == 0 { 0.0 } else { reached as f64 / active as f64 }
    }

    // ── Composite weight ─────────────────────────────────────────────────────

    pub fn weight_one(&self, rule: &ConstraintRule, rule_index: usize, all_rules: &[ConstraintRule]) -> RuleSemanticWeight {
        let proof    = self.necessity_proof(rule, rule_index, all_rules);
        let causal   = self.causal_importance(rule, rule_index, all_rules);
        let structural = Self::structural_significance(rule);
        let reach    = self.reachability(rule, rule_index, all_rules);
        let w = (0.35 * proof.necessity_score + 0.35 * causal + 0.20 * structural + 0.10 * reach).clamp(0.0, 1.0);
        let recommendation = match proof.necessity_class {
            NecessityClass::Critical => if causal < 0.01 { RuleRecommendation::Strengthen } else { RuleRecommendation::Preserve },
            NecessityClass::Observational => RuleRecommendation::DocumentationOnly,
            NecessityClass::Redundant => if causal < 0.01 { RuleRecommendation::RemoveCandidate } else { RuleRecommendation::Simplifiable },
        };
        RuleSemanticWeight {
            rule_name: rule.name.to_string(),
            necessity: proof.necessity_score, necessity_class: proof.necessity_class,
            causal_importance: causal, structural_significance: structural,
            reachability: reach, semantic_weight: w, recommendation,
        }
    }

    pub fn analyze(&self, rule_set: &RuleSet) -> SemanticWeightReport {
        let weights: Vec<RuleSemanticWeight> = rule_set.rules.iter().enumerate()
            .map(|(i, r)| self.weight_one(r, i, &rule_set.rules))
            .collect();
        let avg = if weights.is_empty() { 0.0 }
            else { weights.iter().map(|w| w.semantic_weight).sum::<f64>() / weights.len() as f64 };
        let critical_count  = weights.iter().filter(|w| w.necessity_class == NecessityClass::Critical).count();
        let redundant_count = weights.iter().filter(|w| w.necessity_class == NecessityClass::Redundant).count();
        let remove_candidates = weights.iter().filter(|w| w.recommendation == RuleRecommendation::RemoveCandidate).map(|w| w.rule_name.clone()).collect();
        let simplifiable     = weights.iter().filter(|w| w.recommendation == RuleRecommendation::Simplifiable).map(|w| w.rule_name.clone()).collect();
        SemanticWeightReport { weights, average_weight: avg, critical_rule_count: critical_count, redundant_rule_count: redundant_count, remove_candidates, simplifiable_rules: simplifiable }
    }
}

impl Default for SemanticWeightLayer { fn default() -> Self { Self::new() } }

// ─── SemanticWeightReport ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticWeightReport {
    pub weights: Vec<RuleSemanticWeight>,
    pub average_weight: f64,
    pub critical_rule_count: usize,
    pub redundant_rule_count: usize,
    pub remove_candidates: Vec<String>,
    pub simplifiable_rules: Vec<String>,
}
impl SemanticWeightReport {
    pub fn top_n(&self, n: usize) -> Vec<&RuleSemanticWeight> {
        let mut v: Vec<&RuleSemanticWeight> = self.weights.iter().collect();
        v.sort_by(|a, b| b.semantic_weight.partial_cmp(&a.semantic_weight).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(n); v
    }
    pub fn safe_to_remove(&self) -> Vec<&RuleSemanticWeight> {
        self.weights.iter().filter(|w| w.recommendation == RuleRecommendation::RemoveCandidate).collect()
    }
    pub fn critical_floor(&self) -> f64 {
        self.weights.iter().filter(|w| w.necessity_class == NecessityClass::Critical)
            .map(|w| w.semantic_weight).fold(f64::MAX, f64::min).min(1.0)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;
    use crate::{constraint_algebra::canonical_constraint_rules, proof_certificate::{CertificateBuilder, ExecutionTrace}};

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
        for (from, input, to) in steps { if let Some(c) = b.build(from, input, to) { t.push(c); } }
        t
    }

    fn layer_with_pipeline() -> SemanticWeightLayer {
        let mut l = SemanticWeightLayer::new(); l.observe(&pipeline_trace()); l
    }

    #[test]
    fn pipeline_advance_is_critical() {
        let rs = canonical_constraint_rules();
        let layer = SemanticWeightLayer::new();
        let idx = rs.rules.iter().position(|r| r.name == "pipeline-advance").unwrap();
        let proof = layer.necessity_proof(&rs.rules[idx], idx, &rs.rules);
        assert!(proof.is_critical(), "pipeline-advance must be Critical: {:?}", proof.necessity_class);
        assert!(!proof.critical_cells.is_empty());
    }

    #[test]
    fn no_rule_is_globally_redundant_in_canonical() {
        let rs = canonical_constraint_rules();
        let layer = SemanticWeightLayer::new();
        let mut redundant = Vec::new();
        for (i, rule) in rs.rules.iter().enumerate() {
            let proof = layer.necessity_proof(rule, i, &rs.rules);
            if proof.is_redundant() { redundant.push(rule.name); }
        }
        assert!(redundant.is_empty(), "canonical rules must not be redundant: {redundant:?}");
    }

    #[test]
    fn sealed_absorbing_is_not_redundant() {
        let rs = canonical_constraint_rules();
        let layer = SemanticWeightLayer::new();
        let idx = rs.rules.iter().position(|r| r.name == "sealed-absorbs-all").unwrap();
        let proof = layer.necessity_proof(&rs.rules[idx], idx, &rs.rules);
        assert_ne!(proof.necessity_class, NecessityClass::Redundant);
    }

    #[test]
    fn pipeline_advance_has_high_causal_importance() {
        let rs = canonical_constraint_rules();
        let layer = layer_with_pipeline();
        let idx = rs.rules.iter().position(|r| r.name == "pipeline-advance").unwrap();
        let c = layer.causal_importance(&rs.rules[idx], idx, &rs.rules);
        assert!(c > 0.5, "pipeline-advance must explain >50% of trace: {c:.3}");
    }

    #[test]
    fn faulted_absorbing_zero_causal_for_happy_path() {
        let rs = canonical_constraint_rules();
        let layer = layer_with_pipeline();
        let idx = rs.rules.iter().position(|r| r.name == "faulted-absorbs-non-recovery").unwrap();
        let c = layer.causal_importance(&rs.rules[idx], idx, &rs.rules);
        assert_eq!(c, 0.0, "faulted-absorbing must have 0 causal for fault-free trace: {c}");
    }

    #[test]
    fn structural_significance_scales_with_coverage() {
        let rs = canonical_constraint_rules();
        let sealed   = rs.rules.iter().find(|r| r.name == "sealed-absorbs-all").unwrap();
        let pipeline = rs.rules.iter().find(|r| r.name == "pipeline-advance").unwrap();
        let genesis  = rs.rules.iter().find(|r| r.name == "genesis-init").unwrap();
        let s_large  = SemanticWeightLayer::structural_significance(sealed);
        let s_medium = SemanticWeightLayer::structural_significance(pipeline);
        let s_tiny   = SemanticWeightLayer::structural_significance(genesis);
        assert!(s_large >= s_medium, "large >= medium: {s_large} vs {s_medium}");
        assert!(s_medium >= s_tiny,  "medium >= tiny: {s_medium} vs {s_tiny}");
    }

    #[test]
    fn full_analysis_on_canonical() {
        let rs = canonical_constraint_rules();
        let layer = layer_with_pipeline();
        let report = layer.analyze(&rs);
        assert_eq!(report.weights.len(), rs.rule_count());
        assert!(report.critical_rule_count > 0, "must have critical rules");
        assert!(report.average_weight > 0.0);
        println!("Critical={} Redundant={} Avg={:.3}", report.critical_rule_count, report.redundant_rule_count, report.average_weight);
    }

    #[test]
    fn pipeline_advance_in_top3() {
        let rs = canonical_constraint_rules();
        let layer = layer_with_pipeline();
        let report = layer.analyze(&rs);
        let names: Vec<&str> = report.top_n(3).iter().map(|w| w.rule_name.as_str()).collect();
        println!("Top 3: {names:?}");
        assert!(names.contains(&"pipeline-advance"), "pipeline-advance must be top 3: {names:?}");
    }

    #[test]
    fn critical_rules_never_recommended_for_removal() {
        let rs = canonical_constraint_rules();
        let layer = layer_with_pipeline();
        let report = layer.analyze(&rs);
        for w in &report.weights {
            if w.necessity_class == NecessityClass::Critical {
                assert!(
                    w.recommendation == RuleRecommendation::Preserve || w.recommendation == RuleRecommendation::Strengthen,
                    "critical rule '{}' must not be RemoveCandidate: {:?}", w.rule_name, w.recommendation
                );
            }
        }
    }

    #[test]
    fn critical_rules_anchored_in_transition_table() {
        let rs = canonical_constraint_rules();
        let layer = SemanticWeightLayer::new();
        for (i, rule) in rs.rules.iter().enumerate() {
            let proof = layer.necessity_proof(rule, i, &rs.rules);
            if proof.is_critical() {
                let has_table_cell = proof.critical_cells.iter().any(|&(ph, inp)| TRANSITION_TABLE.iter().any(|e| e.from == ph && e.on == inp));
                assert!(has_table_cell, "critical rule '{}' must cover a table entry", rule.name);
            }
        }
    }

    #[test]
    fn causal_importance_sums_to_one() {
        let rs = canonical_constraint_rules();
        let mut layer = SemanticWeightLayer::new();
        layer.observe(&pipeline_trace());
        let total: f64 = rs.rules.iter().enumerate()
            .map(|(i, r)| layer.causal_importance(r, i, &rs.rules))
            .sum();
        assert!((total - 1.0).abs() < 0.001, "causal importances must sum to 1.0: {total:.4}");
    }
}
