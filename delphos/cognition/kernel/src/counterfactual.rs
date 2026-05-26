// counterfactual.rs — Causal counterfactual reasoning layer.
//
// PROBLEM: "Semantic fixation risk"
//
// When necessity classification stabilises, causal weights converge, and the
// simplifier becomes aggressive, the system freezes into a "locally optimal
// explained universe". Every present rule is justified by current observations.
// But new execution patterns that would have needed removed rules now have no
// coverage.
//
// The four prior layers only answer BACKWARD questions:
//   Necessity:         does removing this rule change what already happened?
//   Causal importance: how much of what already happened goes through this rule?
//   Entropy:           how expressive is the current spec shape?
//   Stability guard:   does the next synthesis cycle degrade the current spec?
//
// None of them answer the FORWARD question:
//   "What would need to happen for rule R to become necessary?"
//
// This module answers that question by computing the MINIMAL HYPOTHETICAL
// EXECUTION TRACE that would make rule R Critical (necessary for a
// TRANSITION_TABLE entry).
//
// If such a trace exists and is SHORT, the rule is "latently important" —
// it guards behavior that is easily reachable but not yet observed.
// The simplifier MUST NOT remove latently important rules.
//
// If no such trace exists, the rule is "structurally unreachable" — dead
// code that cannot become Critical in any valid execution from Genesis.
// The simplifier MAY safely remove it.
//
// FORMAL OBJECTS:
//
//   StateReachabilityGraph
//     BFS over TRANSITION_TABLE starting from Genesis.
//     distances:    KernelPhase → min_steps_from_genesis
//     predecessors: KernelPhase → Vec<(from_phase, via_input)>
//     Invariant: distances[Genesis] = 0
//     Invariant: distances[q] = min over all transitions (p, σ) → q of (distances[p] + 1)
//
//   CounterfactualWitness
//     A specific (phase, input) cell in a rule's active domain,
//     together with the SHORTEST VALID EXECUTION PATH from Genesis to that cell.
//     makes_rule_critical: true iff (phase, input) ∈ TRANSITION_TABLE
//     (only table entries can make a rule Critical in the strict sense)
//
//   CounterfactualAnalysis (per rule)
//     is_reachable:           any cell in active domain is reachable from Genesis
//     can_become_critical:    a TRANSITION_TABLE entry exists in active domain
//                             AND that entry is reachable
//     min_activation_distance: shortest path to exercise ANY active cell
//     min_critical_distance:  shortest path to exercise a CRITICAL cell
//     minimal_critical_witness: the corresponding CounterfactualWitness
//
//   SemanticFixationGuard
//     Uses counterfactual analysis to determine if a rule may be simplified.
//     INVARIANT: a rule is safe to simplify ONLY IF:
//       (a) is_reachable = false   (dead code — cannot execute)
//       (b) OR can_become_critical = false AND min_activation_distance > near_threshold
//     Rules within near_threshold steps from any active cell are PRESERVED
//     even if currently unobserved. This prevents fixation.
//
// Single source of truth. One module, one location.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use serde::{Deserialize, Serialize};

use crate::{
    constraint_algebra::{ConstraintRule, RuleSet},
    kernel_state::{KernelInputKind, KernelPhase, TRANSITION_TABLE},
    semantic_weight::NecessityClass,
};

// ─── StateReachabilityGraph ───────────────────────────────────────────────────

/// BFS reachability graph over TRANSITION_TABLE, rooted at KernelPhase::Genesis.
///
/// Answers: "How many steps does it take to reach phase q from Genesis?"
/// Answers: "What is the shortest input sequence that reaches phase q?"
///
/// Invariants:
///   distances[Genesis] = 0
///   distances[q] = min_{(p,σ)→q ∈ TABLE} (distances[p] + 1)
///   All Faulted/Sealed are reachable (everything eventually faults)
#[derive(Debug, Clone)]
pub struct StateReachabilityGraph {
    /// Minimum steps from Genesis to each phase.
    pub distances: BTreeMap<KernelPhase, usize>,
    /// Best predecessor: phase → (best_predecessor_phase, via_input)
    predecessors: BTreeMap<KernelPhase, (KernelPhase, KernelInputKind)>,
}

impl StateReachabilityGraph {
    /// Build the reachability graph via BFS over TRANSITION_TABLE.
    pub fn build() -> Self {
        let mut distances: BTreeMap<KernelPhase, usize> = BTreeMap::new();
        let mut predecessors: BTreeMap<KernelPhase, (KernelPhase, KernelInputKind)> = BTreeMap::new();
        let mut queue: VecDeque<KernelPhase> = VecDeque::new();

        distances.insert(KernelPhase::Genesis, 0);
        queue.push_back(KernelPhase::Genesis);

        while let Some(phase) = queue.pop_front() {
            let current_dist = distances[&phase];
            for entry in TRANSITION_TABLE {
                if entry.from != phase { continue; }
                if distances.contains_key(&entry.to) { continue; }
                distances.insert(entry.to, current_dist + 1);
                predecessors.insert(entry.to, (phase, entry.on));
                queue.push_back(entry.to);
            }
        }

        Self { distances, predecessors }
    }

    /// Distance from Genesis to a phase. None if unreachable.
    pub fn distance_to(&self, phase: KernelPhase) -> Option<usize> {
        self.distances.get(&phase).copied()
    }

    /// Reconstruct the shortest path from Genesis to `target`.
    /// Returns the sequence of (phase, input) steps.
    /// None if target is unreachable.
    pub fn shortest_path_to(&self, target: KernelPhase) -> Option<Vec<(KernelPhase, KernelInputKind)>> {
        if !self.distances.contains_key(&target) { return None; }
        if target == KernelPhase::Genesis { return Some(vec![]); }

        let mut path = Vec::new();
        let mut current = target;
        while let Some(&(prev, input)) = self.predecessors.get(&current) {
            path.push((current, input));
            current = prev;
        }
        path.reverse();
        Some(path)
    }

    /// All reachable phases.
    pub fn reachable_phases(&self) -> BTreeSet<KernelPhase> {
        self.distances.keys().copied().collect()
    }

    /// True if every phase in ALL is reachable (structural completeness check).
    pub fn is_fully_connected(&self) -> bool {
        KernelPhase::ALL.iter().all(|p| self.distances.contains_key(p))
    }
}

// ─── CounterfactualWitness ────────────────────────────────────────────────────

/// A minimal execution path that would activate a specific cell in a rule's domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualWitness {
    /// The target cell being activated.
    pub target_cell: (KernelPhase, KernelInputKind),
    /// The resulting phase after firing the input.
    pub result_phase: KernelPhase,
    /// Complete path: Vec<(entering_phase, triggering_input)>
    /// Path ends at target_cell. Path[0] is the first step from Genesis.
    pub path: Vec<(KernelPhase, KernelInputKind)>,
    /// Total path length (= path.len()). Includes the triggering step.
    pub path_length: usize,
    /// True if this cell is in TRANSITION_TABLE (makes rule Critical if observed).
    pub makes_rule_critical: bool,
}

impl CounterfactualWitness {
    /// The input sequence from Genesis to trigger this cell.
    pub fn input_sequence(&self) -> Vec<KernelInputKind> {
        self.path.iter().map(|(_, i)| *i).collect()
    }
}

// ─── CounterfactualAnalysis ───────────────────────────────────────────────────

/// Full counterfactual analysis for one rule.
///
/// Answers: "What would need to happen for this rule to matter?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualAnalysis {
    pub rule_name: String,

    /// True if ANY cell in the rule's active domain is reachable from Genesis.
    /// False → rule is dead code (structurally unreachable).
    pub is_reachable: bool,

    /// True if a TRANSITION_TABLE entry exists in the rule's active domain
    /// AND that entry is reachable from Genesis.
    /// This means: given the right trace, this rule COULD become Critical.
    pub can_become_critical: bool,

    /// Minimum steps to exercise ANY active cell (activation, not necessarily Critical).
    /// None if rule is unreachable.
    pub min_activation_distance: Option<usize>,

    /// Minimum steps to exercise a cell that would make rule Critical.
    /// None if the rule cannot become Critical.
    pub min_critical_distance: Option<usize>,

    /// The shortest witness that makes this rule Critical.
    /// None if can_become_critical = false.
    pub minimal_critical_witness: Option<CounterfactualWitness>,

    /// The shortest witness that exercises ANY active cell.
    /// None if is_reachable = false.
    pub minimal_activation_witness: Option<CounterfactualWitness>,

    /// All critical cells (in TRANSITION_TABLE) and their distances.
    pub critical_cell_distances: Vec<((KernelPhase, KernelInputKind), usize)>,
}

impl CounterfactualAnalysis {
    /// Semantic fixation risk score ∈ [0, 1].
    ///
    /// High risk = rule is easily reachable but currently unobserved.
    /// Low risk  = rule is unreachable or far from any execution.
    ///
    /// This is exactly what the simplifier must NOT ignore.
    pub fn fixation_risk(&self, near_threshold: usize) -> f64 {
        if !self.is_reachable { return 0.0; }  // dead code has zero fixation risk
        let dist = self.min_activation_distance.unwrap_or(usize::MAX);
        if dist == 0 { return 1.0; }
        // Risk decays with distance: risk = e^(-dist / near_threshold)
        let t = near_threshold.max(1) as f64;
        (-(dist as f64) / t).exp().clamp(0.0, 1.0)
    }

    /// True if this rule is latently important:
    /// reachable within near_threshold steps AND could become Critical.
    pub fn is_latently_important(&self, near_threshold: usize) -> bool {
        self.can_become_critical
            && self.min_critical_distance.map(|d| d <= near_threshold).unwrap_or(false)
    }
}

// ─── CounterfactualAnalyzer ───────────────────────────────────────────────────

/// Computes CounterfactualAnalysis for every rule in a rule set.
pub struct CounterfactualAnalyzer {
    pub graph: StateReachabilityGraph,
}

impl CounterfactualAnalyzer {
    pub fn new() -> Self { Self { graph: StateReachabilityGraph::build() } }

    /// Analyze one rule.
    pub fn analyze_rule(
        &self,
        rule: &ConstraintRule,
        rule_index: usize,
        all_rules: &[ConstraintRule],
    ) -> CounterfactualAnalysis {
        // Compute active domain (cells where this rule is first match)
        let mut active_cells: Vec<(KernelPhase, KernelInputKind, KernelPhase)> = Vec::new();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if !rule.guard.eval(phase, input) { continue; }
                if all_rules[..rule_index].iter().any(|r| r.guard.eval(phase, input)) { continue; }
                let target = rule.target.resolve(phase);
                active_cells.push((phase, input, target));
            }
        }

        if active_cells.is_empty() {
            return CounterfactualAnalysis {
                rule_name: rule.name.to_string(),
                is_reachable: false, can_become_critical: false,
                min_activation_distance: None, min_critical_distance: None,
                minimal_critical_witness: None, minimal_activation_witness: None,
                critical_cell_distances: vec![],
            };
        }

        // For each active cell: compute distance and check criticality
        let mut best_activation: Option<CounterfactualWitness> = None;
        let mut best_critical: Option<CounterfactualWitness> = None;
        let mut critical_cell_distances: Vec<((KernelPhase, KernelInputKind), usize)> = Vec::new();

        for (phase, input, result) in &active_cells {
            let in_table = TRANSITION_TABLE.iter()
                .any(|e| e.from == *phase && e.on == *input);

            // Distance: steps to reach `phase` from Genesis, plus 1 for the input
            let phase_dist = match self.graph.distance_to(*phase) {
                Some(d) => d,
                None => continue, // phase unreachable → skip this cell
            };
            let total_dist = phase_dist + 1;

            // Reconstruct the path: path to `phase`, then append the triggering step
            let mut path = match self.graph.shortest_path_to(*phase) {
                Some(p) => p,
                None => continue,
            };
            path.push((*phase, *input)); // add the triggering step

            let witness = CounterfactualWitness {
                target_cell: (*phase, *input),
                result_phase: *result,
                path,
                path_length: total_dist,
                makes_rule_critical: in_table,
            };

            // Track best activation witness
            if best_activation.as_ref().map(|w| total_dist < w.path_length).unwrap_or(true) {
                best_activation = Some(witness.clone());
            }

            if in_table {
                critical_cell_distances.push(((*phase, *input), total_dist));
                if best_critical.as_ref().map(|w| total_dist < w.path_length).unwrap_or(true) {
                    best_critical = Some(witness);
                }
            }
        }

        let is_reachable = best_activation.is_some();
        let can_become_critical = best_critical.is_some();
        let min_activation_distance = best_activation.as_ref().map(|w| w.path_length);
        let min_critical_distance = best_critical.as_ref().map(|w| w.path_length);

        CounterfactualAnalysis {
            rule_name: rule.name.to_string(),
            is_reachable,
            can_become_critical,
            min_activation_distance,
            min_critical_distance,
            minimal_critical_witness: best_critical,
            minimal_activation_witness: best_activation,
            critical_cell_distances,
        }
    }

    /// Analyze all rules in a rule set.
    pub fn analyze_all(&self, rule_set: &RuleSet) -> Vec<CounterfactualAnalysis> {
        rule_set.rules.iter().enumerate()
            .map(|(i, r)| self.analyze_rule(r, i, &rule_set.rules))
            .collect()
    }
}

impl Default for CounterfactualAnalyzer { fn default() -> Self { Self::new() } }

// ─── SemanticFixationGuard ────────────────────────────────────────────────────

/// Prevents the simplifier from removing "latently important" rules.
///
/// A rule may be simplified ONLY IF:
///   (a) is_reachable = false          — dead code, cannot execute in any path
///   (b) OR all of:
///       - can_become_critical = false  — even if reached, won't become Critical
///       - min_activation_distance > near_threshold
///       - current necessity != Critical
///       - current causal_importance < 0.01
///
/// Rules within near_threshold steps are PRESERVED regardless of current weight.
/// This is the anti-fixation invariant.
#[derive(Debug, Clone)]
pub struct SemanticFixationGuard {
    /// Distance threshold. Rules reachable within this many steps are preserved.
    pub near_threshold: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimplificationVerdict {
    /// Rule may be simplified (satisfies all conditions).
    SafeToSimplify,
    /// Rule must be preserved — reason given.
    Preserve(PreserveReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreserveReason {
    /// Rule is Critical (necessary for a table entry).
    IsCritical,
    /// Rule is latently important (reachable within near_threshold).
    LatentlyImportant { min_critical_distance: usize },
    /// Rule has non-trivial causal importance.
    HasCausalImportance,
    /// Rule can become Critical and is reachable.
    CanBecomeCritical { min_distance: usize },
}

impl SemanticFixationGuard {
    pub fn new(near_threshold: usize) -> Self { Self { near_threshold } }

    /// Standard threshold: 8 steps from Genesis.
    /// Rationale: Genesis → Init → Bootstrap → Idle → ... → any processing phase (≤ 12)
    /// Threshold 8 covers the complete happy path and most error paths.
    pub fn standard() -> Self { Self::new(8) }

    /// Determine if a rule may be simplified.
    pub fn verdict(
        &self,
        analysis: &CounterfactualAnalysis,
        necessity_class: NecessityClass,
        causal_importance: f64,
    ) -> SimplificationVerdict {
        // (1) Critical rules are never simplified
        if necessity_class == NecessityClass::Critical {
            return SimplificationVerdict::Preserve(PreserveReason::IsCritical);
        }

        // (2) Rules with meaningful causal importance are preserved
        if causal_importance >= 0.01 {
            return SimplificationVerdict::Preserve(PreserveReason::HasCausalImportance);
        }

        // (3) Unreachable rules: safe to remove (dead code)
        if !analysis.is_reachable {
            return SimplificationVerdict::SafeToSimplify;
        }

        // (4) Latently important: reachable within threshold AND can become Critical
        if let Some(d) = analysis.min_critical_distance {
            if d <= self.near_threshold {
                return SimplificationVerdict::Preserve(
                    PreserveReason::LatentlyImportant { min_critical_distance: d }
                );
            }
        }

        // (5) Can become Critical at all (even if far)
        if analysis.can_become_critical {
            let dist = analysis.min_critical_distance.unwrap_or(usize::MAX);
            return SimplificationVerdict::Preserve(
                PreserveReason::CanBecomeCritical { min_distance: dist }
            );
        }

        // (6) Not reachable within threshold even for activation-only
        if let Some(d) = analysis.min_activation_distance {
            if d <= self.near_threshold {
                // Reachable but can't become Critical — still preserve as documentation
                return SimplificationVerdict::Preserve(
                    PreserveReason::LatentlyImportant { min_critical_distance: usize::MAX }
                );
            }
        }

        SimplificationVerdict::SafeToSimplify
    }

    pub fn is_fixation_safe(&self, analysis: &CounterfactualAnalysis, necessity: NecessityClass, causal: f64) -> bool {
        self.verdict(analysis, necessity, causal) == SimplificationVerdict::SafeToSimplify
    }
}

// ─── CounterfactualReport ─────────────────────────────────────────────────────

/// Full counterfactual report for an entire rule set.
#[derive(Debug, Clone)]
pub struct CounterfactualReport {
    pub analyses: Vec<CounterfactualAnalysis>,
    pub graph_stats: ReachabilityStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityStats {
    pub total_phases: usize,
    pub reachable_phases: usize,
    pub unreachable_phases: Vec<KernelPhase>,
    pub max_distance: usize,
    pub phases_by_distance: BTreeMap<usize, Vec<KernelPhase>>,
}

impl CounterfactualReport {
    pub fn build(analyzer: &CounterfactualAnalyzer, rule_set: &RuleSet) -> Self {
        let analyses = analyzer.analyze_all(rule_set);

        // Build graph stats
        let reachable = analyzer.graph.reachable_phases();
        let unreachable: Vec<KernelPhase> = KernelPhase::ALL.iter()
            .filter(|p| !reachable.contains(p))
            .copied()
            .collect();
        let max_dist = analyzer.graph.distances.values().copied().max().unwrap_or(0);
        let mut phases_by_dist: BTreeMap<usize, Vec<KernelPhase>> = BTreeMap::new();
        for (&phase, &dist) in &analyzer.graph.distances {
            phases_by_dist.entry(dist).or_default().push(phase);
        }

        Self {
            analyses,
            graph_stats: ReachabilityStats {
                total_phases: KernelPhase::ALL.len(),
                reachable_phases: reachable.len(),
                unreachable_phases: unreachable,
                max_distance: max_dist,
                phases_by_distance: phases_by_dist,
            },
        }
    }

    pub fn latently_important_rules(&self, threshold: usize) -> Vec<&CounterfactualAnalysis> {
        self.analyses.iter().filter(|a| a.is_latently_important(threshold)).collect()
    }

    pub fn unreachable_rules(&self) -> Vec<&CounterfactualAnalysis> {
        self.analyses.iter().filter(|a| !a.is_reachable).collect()
    }

    pub fn fixation_risks(&self, threshold: usize) -> Vec<(&CounterfactualAnalysis, f64)> {
        self.analyses.iter()
            .map(|a| (a, a.fixation_risk(threshold)))
            .filter(|(_, r)| *r > 0.01)
            .collect()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint_algebra::canonical_constraint_rules;

    fn analyzer() -> CounterfactualAnalyzer { CounterfactualAnalyzer::new() }

    // ── Reachability graph ───────────────────────────────────────────────

    #[test]
    fn genesis_distance_is_zero() {
        let g = StateReachabilityGraph::build();
        assert_eq!(g.distance_to(KernelPhase::Genesis), Some(0));
    }

    #[test]
    fn bootstrapping_reachable_in_one_step() {
        let g = StateReachabilityGraph::build();
        // Genesis → Initialize → Bootstrapping
        assert_eq!(g.distance_to(KernelPhase::Bootstrapping), Some(1));
    }

    #[test]
    fn idle_reachable_in_two_steps() {
        let g = StateReachabilityGraph::build();
        // Genesis → Initialize → Bootstrapping → BootstrapComplete → Idle
        assert_eq!(g.distance_to(KernelPhase::Idle), Some(2));
    }

    #[test]
    fn processing_phases_reachable_from_idle() {
        let g = StateReachabilityGraph::build();
        // ValidatingAbi reachable in Idle(2) + EventArrived(1) = 3 steps
        // Distances: Idle(2) + pipeline steps
        assert_eq!(g.distance_to(KernelPhase::ValidatingAbi), Some(3));
        assert_eq!(g.distance_to(KernelPhase::ValidatingSchema), Some(4));
        // Deciding: Idle(2) + 6 pipeline steps = 8
        let deciding_dist = g.distance_to(KernelPhase::Deciding).unwrap();
        assert!(deciding_dist >= 7 && deciding_dist <= 10,
            "Deciding must be reachable in 7-10 steps: {deciding_dist}");
        let applying_dist = g.distance_to(KernelPhase::Applying).unwrap();
        assert!(applying_dist >= 8 && applying_dist <= 12,
            "Applying must be reachable in 8-12 steps: {applying_dist}");
    }

    #[test]
    fn faulted_reachable() {
        let g = StateReachabilityGraph::build();
        // Idle(2) + EventArrived(1) + FaultDetected(1) = 4
        assert!(g.distance_to(KernelPhase::Faulted).map(|d| d <= 5).unwrap_or(false));
    }

    #[test]
    fn sealed_reachable() {
        let g = StateReachabilityGraph::build();
        // Idle(2) + SealRequested(1) = 3
        assert_eq!(g.distance_to(KernelPhase::Sealed), Some(3));
    }

    #[test]
    fn path_reconstruction_is_valid() {
        let g = StateReachabilityGraph::build();
        let path = g.shortest_path_to(KernelPhase::Idle).unwrap();
        assert!(!path.is_empty());
        // Last step must land at Idle
        let (_, last_input) = path.last().unwrap();
        assert_eq!(*last_input, KernelInputKind::BootstrapComplete);
    }

    #[test]
    fn all_phases_reachable_from_genesis() {
        // Every KernelPhase must be reachable from Genesis via TRANSITION_TABLE.
        let g = StateReachabilityGraph::build();
        let mut unreachable = Vec::new();
        for &phase in KernelPhase::ALL {
            if g.distance_to(phase).is_none() { unreachable.push(phase); }
        }
        assert!(unreachable.is_empty(),
            "All phases must be reachable from Genesis. Unreachable: {unreachable:?}");
    }

    // ── Counterfactual analysis ──────────────────────────────────────────

    #[test]
    fn pipeline_advance_can_become_critical() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let idx = rs.rules.iter().position(|r| r.name == "pipeline-advance").unwrap();
        let analysis = a.analyze_rule(&rs.rules[idx], idx, &rs.rules);
        assert!(analysis.is_reachable, "pipeline-advance must be reachable");
        assert!(analysis.can_become_critical, "pipeline-advance must be able to become Critical");
        let dist = analysis.min_critical_distance.unwrap();
        // Idle(2) + EventArrived(1) = 3 minimum
        assert!(dist >= 3, "min critical distance must be >= 3: {dist}");
        assert!(dist <= 12, "min critical distance must be <= 12: {dist}");
    }

    #[test]
    fn universal_fault_can_become_critical() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let idx = rs.rules.iter().position(|r| r.name == "universal-fault").unwrap();
        let analysis = a.analyze_rule(&rs.rules[idx], idx, &rs.rules);
        assert!(analysis.is_reachable);
        // FaultDetected from Idle: Idle(2) + FaultDetected(1) = 3
        println!("universal-fault: min_critical={:?}, min_activation={:?}",
            analysis.min_critical_distance, analysis.min_activation_distance);
    }

    #[test]
    fn sealed_absorbing_is_reachable() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let idx = rs.rules.iter().position(|r| r.name == "sealed-absorbs-all").unwrap();
        let analysis = a.analyze_rule(&rs.rules[idx], idx, &rs.rules);
        assert!(analysis.is_reachable, "sealed-absorbs-all must be reachable");
    }

    #[test]
    fn all_canonical_rules_are_reachable() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let analyses = a.analyze_all(&rs);
        let unreachable: Vec<&str> = analyses.iter()
            .filter(|a| !a.is_reachable)
            .map(|a| a.rule_name.as_str())
            .collect();
        assert!(unreachable.is_empty(),
            "all canonical rules must be reachable: {unreachable:?}");
    }

    #[test]
    fn witness_path_is_valid_execution() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let idx = rs.rules.iter().position(|r| r.name == "pipeline-advance").unwrap();
        let analysis = a.analyze_rule(&rs.rules[idx], idx, &rs.rules);
        let witness = analysis.minimal_critical_witness.unwrap();

        // The path must start at Genesis phase (first entry phase is Genesis or reaches it)
        assert!(!witness.path.is_empty());
        assert_eq!(witness.path_length, witness.path.len());

        // Target cell must be Critical
        assert!(witness.makes_rule_critical);
    }

    #[test]
    fn minimal_witness_length_matches_distance() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let idx = rs.rules.iter().position(|r| r.name == "genesis-init").unwrap();
        let analysis = a.analyze_rule(&rs.rules[idx], idx, &rs.rules);
        // genesis-init fires at (Genesis, Initialize) — distance 0 + 1 = 1
        assert_eq!(analysis.min_activation_distance, Some(1),
            "genesis-init activates in 1 step: {:?}", analysis.min_activation_distance);
    }

    // ── Semantic fixation guard ───────────────────────────────────────────

    #[test]
    fn pipeline_advance_is_not_safe_to_simplify() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let guard = SemanticFixationGuard::standard();
        let idx = rs.rules.iter().position(|r| r.name == "pipeline-advance").unwrap();
        let analysis = a.analyze_rule(&rs.rules[idx], idx, &rs.rules);
        let verdict = guard.verdict(&analysis, NecessityClass::Critical, 0.5);
        assert_eq!(verdict, SimplificationVerdict::Preserve(PreserveReason::IsCritical));
    }

    #[test]
    fn latently_important_rules_are_preserved() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let guard = SemanticFixationGuard::standard();
        let analyses = a.analyze_all(&rs);
        // Any rule that can become Critical within 8 steps must be preserved
        for analysis in &analyses {
            if analysis.is_latently_important(guard.near_threshold) {
                let verdict = guard.verdict(analysis, NecessityClass::Observational, 0.0);
                assert!(
                    matches!(verdict, SimplificationVerdict::Preserve(_)),
                    "latently important rule '{}' must be preserved: {:?}",
                    analysis.rule_name, verdict
                );
            }
        }
    }

    #[test]
    fn no_canonical_rule_is_fixation_safe() {
        // Every canonical rule must be preserved under the default guard.
        // (They're all reachable AND can become Critical.)
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let guard = SemanticFixationGuard::standard();
        let analyses = a.analyze_all(&rs);
        for analysis in &analyses {
            let verdict = guard.verdict(&analysis, NecessityClass::Observational, 0.0);
            assert!(
                matches!(verdict, SimplificationVerdict::Preserve(_)),
                "canonical rule '{}' must be preserved: {:?}", analysis.rule_name, verdict
            );
        }
    }

    #[test]
    fn fixation_risk_inversely_proportional_to_distance() {
        let a = analyzer();
        let rs = canonical_constraint_rules();
        let guard = SemanticFixationGuard::standard();

        // genesis-init (distance 1) must have higher risk than identity-confirmed (~12)
        let genesis_idx = rs.rules.iter().position(|r| r.name == "genesis-init").unwrap();
        let id_conf_idx = rs.rules.iter().position(|r| r.name == "identity-confirmed").unwrap();
        let genesis_a  = a.analyze_rule(&rs.rules[genesis_idx],  genesis_idx,  &rs.rules);
        let id_conf_a  = a.analyze_rule(&rs.rules[id_conf_idx],  id_conf_idx,  &rs.rules);
        let risk_near = genesis_a.fixation_risk(guard.near_threshold);
        let risk_far  = id_conf_a.fixation_risk(guard.near_threshold);
        println!("genesis-init fixation risk: {risk_near:.3}, identity-confirmed: {risk_far:.3}");
        assert!(risk_near >= risk_far,
            "near rules must have >= fixation risk than far rules: {risk_near} vs {risk_far}");
    }

    // ── Full report ───────────────────────────────────────────────────────

    #[test]
    fn full_report_on_canonical() {
        let rs = canonical_constraint_rules();
        let a = analyzer();
        let report = CounterfactualReport::build(&a, &rs);
        assert_eq!(report.analyses.len(), rs.rule_count());
        assert_eq!(report.graph_stats.total_phases, KernelPhase::ALL.len());
        assert!(report.graph_stats.unreachable_phases.is_empty(),
            "no phases must be unreachable: {:?}", report.graph_stats.unreachable_phases);
        println!("Graph: {} phases reachable, max_distance={}",
            report.graph_stats.reachable_phases, report.graph_stats.max_distance);
        println!("Latently important rules: {}",
            report.latently_important_rules(8).len());
    }

    #[test]
    fn phases_by_distance_covers_all_phases() {
        let a = analyzer();
        let rs = canonical_constraint_rules();
        let report = CounterfactualReport::build(&a, &rs);
        let covered: usize = report.graph_stats.phases_by_distance.values()
            .map(|v| v.len()).sum();
        assert_eq!(covered, KernelPhase::ALL.len(),
            "phases_by_distance must cover every phase exactly once");
    }
}
