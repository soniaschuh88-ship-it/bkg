// constraint_algebra.rs — Symbolic constraint expressions + rule algebra.
//
// The rules in rule_engine.rs were expressed as Rust data structures.
// They were verifiable but not ALGEBRAICALLY ANALYZABLE:
//   - Can rule R₁ subsume rule R₂? (is R₁ strictly more general?)
//   - Do rules R₁ and R₂ conflict? (same (phase,input), different targets?)
//   - What is the minimal equivalent rule set?
//   - Is the pipeline acyclic? (no reachable cycle except the Emitting→Idle reset)
//
// FIX: Express rules as ConstraintExpr — a symbolic predicate algebra
// over the finite domain Q×Σ (18 phases × 29 inputs = 522 cells).
// Evaluation is by exhaustive enumeration — correct but not expensive
// given the domain size.
//
// Operations:
//   eval(phase, input) → bool          predicate evaluation
//   and / or / not                     Boolean algebra
//   subsumes(other)                    R₁ subsumes R₂ iff dom(R₁) ⊇ dom(R₂)
//   conflicts_with(other, table)       same domain, different targets
//   extension()                        all (phase,input) where predicate holds
//
// The RuleSet builds on this to provide:
//   verify_consistency()               no two rules conflict
//   find_redundant()                   rules subsumed by others
//   minimize()                         remove subsumed rules
//   prove_pipeline_acyclic()           processing phases form a DAG
//
// Single source of truth for δ specification.

use std::collections::{BTreeMap, BTreeSet};
use crate::kernel_state::{KernelInputKind, KernelPhase};

// ─── ConstraintExpr ──────────────────────────────────────────────────────────

/// A symbolic predicate over (KernelPhase, KernelInputKind).
/// Evaluated over the finite 522-cell domain.
#[derive(Debug, Clone)]
pub enum ConstraintExpr {
    // ── Tautologies / contradictions ─────────────────────────────────────
    /// Always true.
    True,
    /// Always false.
    False,

    // ── Phase predicates ──────────────────────────────────────────────────
    /// phase == p
    PhaseEq(KernelPhase),
    /// phase ∈ {p₁, p₂, ...}
    PhaseIn(Vec<KernelPhase>),
    /// phase.is_processing()
    PhaseIsProcessing,
    /// phase.is_terminal()
    PhaseIsTerminal,
    /// phase.is_replaying()
    PhaseIsReplaying,

    // ── Input predicates ──────────────────────────────────────────────────
    /// input == σ
    InputEq(KernelInputKind),
    /// input ∈ {σ₁, σ₂, ...}
    InputIn(Vec<KernelInputKind>),

    // ── Boolean algebra ───────────────────────────────────────────────────
    /// e₁ ∧ e₂
    And(Box<ConstraintExpr>, Box<ConstraintExpr>),
    /// e₁ ∨ e₂
    Or(Box<ConstraintExpr>, Box<ConstraintExpr>),
    /// ¬e
    Not(Box<ConstraintExpr>),
}

impl ConstraintExpr {
    /// Evaluate the predicate for a specific (phase, input) pair.
    pub fn eval(&self, phase: KernelPhase, input: KernelInputKind) -> bool {
        match self {
            Self::True  => true,
            Self::False => false,

            Self::PhaseEq(p)     => phase == *p,
            Self::PhaseIn(ps)    => ps.contains(&phase),
            Self::PhaseIsProcessing => phase.is_processing(),
            Self::PhaseIsTerminal   => phase.is_terminal(),
            Self::PhaseIsReplaying  => phase.is_replaying(),

            Self::InputEq(s)     => input == *s,
            Self::InputIn(ss)    => ss.contains(&input),

            Self::And(a, b) => a.eval(phase, input) && b.eval(phase, input),
            Self::Or(a, b)  => a.eval(phase, input) || b.eval(phase, input),
            Self::Not(e)    => !e.eval(phase, input),
        }
    }

    /// Compute the extension: all (phase, input) pairs where the predicate holds.
    pub fn extension(&self) -> BTreeSet<(KernelPhase, KernelInputKind)> {
        let mut result = BTreeSet::new();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if self.eval(phase, input) {
                    result.insert((phase, input));
                }
            }
        }
        result
    }

    /// Cardinality of the extension (how many (phase, input) cells this covers).
    pub fn cardinality(&self) -> usize { self.extension().len() }

    /// Is this predicate a tautology (always true)?
    pub fn is_tautology(&self) -> bool {
        KernelPhase::ALL.iter().all(|&p| KernelInputKind::ALL.iter().all(|&i| self.eval(p, i)))
    }

    /// Is this predicate a contradiction (always false)?
    pub fn is_contradiction(&self) -> bool {
        KernelPhase::ALL.iter().all(|&p| KernelInputKind::ALL.iter().all(|&i| !self.eval(p, i)))
    }

    // ── Algebraic constructors ────────────────────────────────────────────

    pub fn and(self, other: ConstraintExpr) -> ConstraintExpr {
        ConstraintExpr::And(Box::new(self), Box::new(other))
    }
    pub fn or(self, other: ConstraintExpr) -> ConstraintExpr {
        ConstraintExpr::Or(Box::new(self), Box::new(other))
    }
    pub fn negate(self) -> ConstraintExpr {
        ConstraintExpr::Not(Box::new(self))
    }

    /// Does this predicate's extension INCLUDE other's extension?
    /// R₁ subsumes R₂ iff ∀x: R₂(x) → R₁(x)
    pub fn subsumes(&self, other: &ConstraintExpr) -> bool {
        // R₁ subsumes R₂ iff (R₂ ∧ ¬R₁) is empty
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if other.eval(phase, input) && !self.eval(phase, input) {
                    return false; // R₂ holds but R₁ doesn't → not subsumed
                }
            }
        }
        true
    }

    /// Intersection with another predicate.
    pub fn intersects(&self, other: &ConstraintExpr) -> bool {
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if self.eval(phase, input) && other.eval(phase, input) {
                    return true;
                }
            }
        }
        false
    }
}

// ─── ConstraintTarget ────────────────────────────────────────────────────────

/// The target of a constraint rule — what phase to transition to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintTarget {
    /// A specific target phase.
    Phase(KernelPhase),
    /// Self-loop: stay in the same phase.
    Self_,
    /// next_processing_phase() — the next step in the ordered pipeline.
    NextInPipeline,
}

impl ConstraintTarget {
    pub fn resolve(&self, current_phase: KernelPhase) -> KernelPhase {
        match self {
            Self::Phase(p) => *p,
            Self::Self_    => current_phase,
            Self::NextInPipeline =>
                current_phase.next_processing_phase().unwrap_or(KernelPhase::Faulted),
        }
    }
}

// ─── ConstraintRule ──────────────────────────────────────────────────────────

/// A rule expressed as (name, guard, target) where guard is a ConstraintExpr.
#[derive(Debug, Clone)]
pub struct ConstraintRule {
    pub name: &'static str,
    /// When this rule fires: the (phase, input) predicate.
    pub guard: ConstraintExpr,
    /// What target phase to produce.
    pub target: ConstraintTarget,
}

impl ConstraintRule {
    pub fn new(name: &'static str, guard: ConstraintExpr, target: ConstraintTarget) -> Self {
        Self { name, guard, target }
    }

    /// All (phase, input, target_phase) triples this rule generates.
    pub fn entries(&self) -> Vec<(KernelPhase, KernelInputKind, KernelPhase)> {
        self.guard.extension().into_iter()
            .map(|(phase, input)| (phase, input, self.target.resolve(phase)))
            .collect()
    }

    /// The domain of this rule (set of (phase,input) where guard holds).
    pub fn domain(&self) -> BTreeSet<(KernelPhase, KernelInputKind)> {
        self.guard.extension()
    }

    /// Does rule self subsume rule other?
    /// (self fires everywhere other fires, with same or broader coverage)
    pub fn subsumes_guard_of(&self, other: &ConstraintRule) -> bool {
        self.guard.subsumes(&other.guard)
    }

    /// Does this rule conflict with another rule?
    /// (both fire on some input, but produce different targets)
    pub fn conflicts_with(&self, other: &ConstraintRule) -> Vec<(KernelPhase, KernelInputKind)> {
        let mut conflicts = Vec::new();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                if self.guard.eval(phase, input) && other.guard.eval(phase, input) {
                    let my_target = self.target.resolve(phase);
                    let other_target = other.target.resolve(phase);
                    if my_target != other_target {
                        conflicts.push((phase, input));
                    }
                }
            }
        }
        conflicts
    }
}

// ─── RuleSet ─────────────────────────────────────────────────────────────────

/// A set of ConstraintRules forming a complete specification of δ.
pub struct RuleSet {
    pub rules: Vec<ConstraintRule>,
}

#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub rule_a: &'static str,
    pub rule_b: &'static str,
    pub conflicting_cells: Vec<(KernelPhase, KernelInputKind)>,
}

#[derive(Debug, Clone)]
pub struct ConsistencyReport {
    pub rule_count: usize,
    pub conflicts: Vec<ConflictReport>,
    pub redundant_rules: Vec<(&'static str, &'static str)>, // (subsumed, subsuming)
    pub coverage: usize,  // cells covered (distinct (phase,input) pairs)
    pub total_cells: usize,
    pub is_consistent: bool,
}

#[derive(Debug, Clone)]
pub struct AcyclicityProof {
    pub is_acyclic: bool,
    /// Any processing phase cycles found (excluding the Emitting→Idle reset arc).
    pub cycles: Vec<Vec<KernelPhase>>,
    pub forward_only_phases: Vec<KernelPhase>,
}

impl RuleSet {
    pub fn new(rules: Vec<ConstraintRule>) -> Self { Self { rules } }

    /// Synthesize the complete transition table from this rule set.
    /// Priority: first matching rule wins.
    pub fn synthesize(&self) -> BTreeMap<(KernelPhase, KernelInputKind), KernelPhase> {
        let mut table: BTreeMap<(KernelPhase, KernelInputKind), KernelPhase> = BTreeMap::new();
        for rule in &self.rules {
            for (phase, input, target) in rule.entries() {
                table.entry((phase, input)).or_insert(target);
            }
        }
        table
    }

    /// Evaluate δ(phase, input) using this rule set.
    /// First matching rule wins; undefined → KernelPhase::Faulted.
    pub fn delta(&self, phase: KernelPhase, input: KernelInputKind) -> KernelPhase {
        for rule in &self.rules {
            if rule.guard.eval(phase, input) {
                return rule.target.resolve(phase);
            }
        }
        KernelPhase::Faulted
    }

    /// Check consistency: no two rules produce different targets for the same (phase, input).
    pub fn verify_consistency(&self) -> ConsistencyReport {
        let mut conflicts = Vec::new();
        let n = self.rules.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let conflicting_cells = self.rules[i].conflicts_with(&self.rules[j]);
                if !conflicting_cells.is_empty() {
                    conflicts.push(ConflictReport {
                        rule_a: self.rules[i].name,
                        rule_b: self.rules[j].name,
                        conflicting_cells,
                    });
                }
            }
        }

        // Find redundant rules: rule i is subsumed by rule j if:
        // - rule j subsumes rule i's guard
        // - rule j produces the same target for all cells in rule i's domain
        let mut redundant = Vec::new();
        for i in 0..n {
            for j in 0..n {
                if i == j { continue; }
                if self.rules[j].subsumes_guard_of(&self.rules[i]) {
                    // Check targets match on the intersection
                    let is_same_target = self.rules[i].domain().iter().all(|&(phase, _input)| {
                        self.rules[i].target.resolve(phase) == self.rules[j].target.resolve(phase)
                    });
                    if is_same_target {
                        redundant.push((self.rules[i].name, self.rules[j].name));
                    }
                }
            }
        }

        let coverage = self.synthesize().len();
        let total_cells = KernelPhase::ALL.len() * KernelInputKind::ALL.len();
        let is_consistent = conflicts.is_empty();

        ConsistencyReport { rule_count: n, conflicts, redundant_rules: redundant, coverage, total_cells, is_consistent }
    }

    /// Prove the processing pipeline is acyclic (excluding the Emitting→Idle reset arc).
    /// Acyclic means: no processing phase can reach itself via normal processing inputs.
    #[allow(unused_variables)]
    pub fn prove_pipeline_acyclic(&self) -> AcyclicityProof {
        use KernelPhase::*;
        let processing = [
            ValidatingAbi, ValidatingSchema, ValidatingClock,
            ValidatingCapability, ValidatingCausal, Deciding,
            Applying, Stamping, Emitting,
        ];

        // The valid forward order
        let order: BTreeMap<KernelPhase, usize> = processing.iter().enumerate()
            .map(|(i, &p)| (p, i))
            .collect();

        let mut cycles = Vec::new();

        // Check: from any processing phase, can we reach a LOWER-INDEXED processing phase?
        // (excluding the explicit Emitting→Idle reset arc)
        for &phase in &processing {
            for &input in KernelInputKind::ALL {
                let target = self.delta(phase, input);
                if let Some(&target_idx) = order.get(&target) {
                    if let Some(&phase_idx) = order.get(&phase) {
                        // A backward transition in the pipeline is a cycle
                        if target_idx < phase_idx {
                            // Is this the Emitting→Idle reset? Idle is NOT in processing array.
                            // (Emitting→Idle is fine — it's the cycle-back, not a processing cycle)
                            cycles.push(vec![phase, target]);
                        }
                    }
                }
            }
        }

        let forward_only: Vec<KernelPhase> = processing.iter()
            .filter(|&&p| !cycles.iter().any(|c| c[0] == p))
            .copied()
            .collect();

        AcyclicityProof {
            is_acyclic: cycles.is_empty(),
            cycles,
            forward_only_phases: forward_only,
        }
    }

    pub fn rule_count(&self) -> usize { self.rules.len() }
}

// ─── Canonical constraint rule set ───────────────────────────────────────────

/// Express the DELPHOS kernel δ as a constraint rule set.
/// This is the algebraic specification — the source of truth.
pub fn canonical_constraint_rules() -> RuleSet {
    use KernelPhase as P;
    use KernelInputKind as I;

    // Helper macros (closures to avoid repetition)
    let phase_eq    = |p| ConstraintExpr::PhaseEq(p);
    let _phase_in    = |ps: &[KernelPhase]| ConstraintExpr::PhaseIn(ps.to_vec());
    let input_eq    = |s| ConstraintExpr::InputEq(s);
    let _input_in    = |ss: &[KernelInputKind]| ConstraintExpr::InputIn(ss.to_vec());
    let and         = |a: ConstraintExpr, b: ConstraintExpr| a.and(b);
    let not         = |e: ConstraintExpr| e.negate();

    RuleSet::new(vec![
        // ── R0: Sealed absorbs all inputs ────────────────────────────────
        ConstraintRule::new(
            "sealed-absorbs-all",
            phase_eq(P::Sealed),
            ConstraintTarget::Self_,
        ),

        // ── R1: Faulted absorbs all except RecoveryAttempted ─────────────
        ConstraintRule::new(
            "faulted-absorbs-non-recovery",
            and(
                phase_eq(P::Faulted),
                not(input_eq(I::RecoveryAttempted)),
            ),
            ConstraintTarget::Self_,
        ),

        // ── R2: Faulted + RecoveryAttempted → Recovering ─────────────────
        ConstraintRule::new(
            "faulted-recovery-escape",
            and(phase_eq(P::Faulted), input_eq(I::RecoveryAttempted)),
            ConstraintTarget::Phase(P::Recovering),
        ),

        // ── R3: FaultDetected from any non-terminal phase → Faulted ──────
        ConstraintRule::new(
            "universal-fault",
            and(
                ConstraintExpr::PhaseIsTerminal.negate(),
                input_eq(I::FaultDetected),
            ),
            ConstraintTarget::Phase(P::Faulted),
        ),

        // ── R4: Pipeline advance — union of exact (phase, input) pairs ────────
        // Cross-products over-generalize: (ValidatingAbi, EventArrived) is NOT valid.
        // Only specific pairs advance the pipeline. Express as a disjunction.
        ConstraintRule::new(
            "pipeline-advance",
            // Each pair: phase_eq(p) ∧ input_eq(σ) where (p,σ) is a valid pipeline step
            {
                let steps: &[(KernelPhase, KernelInputKind)] = &[
                    (P::Idle,                 I::EventArrived),
                    (P::ValidatingAbi,        I::AbiValid),
                    (P::ValidatingSchema,     I::SchemaValid),
                    (P::ValidatingClock,      I::ClockValid),
                    (P::ValidatingCapability, I::CapabilityGranted),
                    (P::ValidatingCausal,     I::CausalValid),
                    (P::Deciding,             I::DecisionAllow),
                    (P::Deciding,             I::DecisionTransform),
                    (P::Applying,             I::TransitionApplied),
                    (P::Stamping,             I::ProjectionStamped),
                    (P::Emitting,             I::EmitComplete),
                ];
                steps.iter().fold(ConstraintExpr::False, |acc, &(ph, inp)| {
                    acc.or(phase_eq(ph).and(input_eq(inp)))
                })
            },
            ConstraintTarget::NextInPipeline,
        ),

        // ── R5: Validation rejection — union of exact (phase, failure) pairs ─
        // Each validation phase has exactly ONE failure input. Cross-product wrong.
        ConstraintRule::new(
            "validation-rejection",
            {
                let pairs: &[(KernelPhase, KernelInputKind)] = &[
                    (P::ValidatingAbi,        I::AbiFailed),
                    (P::ValidatingSchema,     I::SchemaFailed),
                    (P::ValidatingClock,      I::ClockFailed),
                    (P::ValidatingCapability, I::CapabilityDenied),
                    (P::ValidatingCausal,     I::CausalFailed),
                    (P::Deciding,             I::DecisionReject),
                ];
                pairs.iter().fold(ConstraintExpr::False, |acc, &(ph, inp)| {
                    acc.or(phase_eq(ph).and(input_eq(inp)))
                })
            },
            ConstraintTarget::Phase(P::Idle),
        ),

        // ── R6: TransitionFailed → Recovering ────────────────────────────
        ConstraintRule::new(
            "transition-failed-to-recovering",
            and(phase_eq(P::Applying), input_eq(I::TransitionFailed)),
            ConstraintTarget::Phase(P::Recovering),
        ),

        // ── R7: Recovery arcs ─────────────────────────────────────────────
        ConstraintRule::new("recovery-self-loop",
            and(phase_eq(P::Recovering), input_eq(I::RecoveryAttempted)),
            ConstraintTarget::Self_),
        ConstraintRule::new("recovery-success",
            and(phase_eq(P::Recovering), input_eq(I::RecoverySucceeded)),
            ConstraintTarget::Phase(P::Idle)),

        // ── R8: Replay arcs ───────────────────────────────────────────────
        ConstraintRule::new("replay-request",
            and(phase_eq(P::Idle), input_eq(I::ReplayRequested)),
            ConstraintTarget::Phase(P::ReplayPending)),
        ConstraintRule::new("replay-start",
            and(phase_eq(P::ReplayPending), input_eq(I::EventArrived)),
            ConstraintTarget::Phase(P::Replaying)),
        ConstraintRule::new("replay-event-self",
            and(phase_eq(P::Replaying), input_eq(I::ReplayEventApplied)),
            ConstraintTarget::Self_),
        ConstraintRule::new("replay-complete",
            and(phase_eq(P::Replaying), input_eq(I::ReplayComplete)),
            ConstraintTarget::Phase(P::VerifyingIdentity)),
        ConstraintRule::new("identity-confirmed",
            and(phase_eq(P::VerifyingIdentity), input_eq(I::IdentityConfirmed)),
            ConstraintTarget::Phase(P::Idle)),
        ConstraintRule::new("identity-diverged",
            and(phase_eq(P::VerifyingIdentity), input_eq(I::IdentityDiverged)),
            ConstraintTarget::Phase(P::Faulted)),

        // ── R9: Lifecycle arcs ────────────────────────────────────────────
        ConstraintRule::new("genesis-init",
            and(phase_eq(P::Genesis), input_eq(I::Initialize)),
            ConstraintTarget::Phase(P::Bootstrapping)),
        ConstraintRule::new("bootstrap-complete",
            and(phase_eq(P::Bootstrapping), input_eq(I::BootstrapComplete)),
            ConstraintTarget::Phase(P::Idle)),
        ConstraintRule::new("seal-from-idle",
            and(phase_eq(P::Idle), input_eq(I::SealRequested)),
            ConstraintTarget::Phase(P::Sealed)),
        ConstraintRule::new("seal-from-replay",
            and(phase_eq(P::Replaying), input_eq(I::SealRequested)),
            ConstraintTarget::Phase(P::Sealed)),
    ])
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ruleset() -> RuleSet { canonical_constraint_rules() }

    // ── Algebraic properties ────────────────────────────────────────────

    #[test]
    fn true_is_tautology() {
        assert!(ConstraintExpr::True.is_tautology());
    }

    #[test]
    fn false_is_contradiction() {
        assert!(ConstraintExpr::False.is_contradiction());
    }

    #[test]
    fn phase_eq_and_not_is_contradiction() {
        let e = ConstraintExpr::PhaseEq(KernelPhase::Idle)
            .and(ConstraintExpr::PhaseEq(KernelPhase::Idle).negate());
        assert!(e.is_contradiction());
    }

    #[test]
    fn terminal_subsumes_sealed() {
        let terminal = ConstraintExpr::PhaseIsTerminal;
        let sealed = ConstraintExpr::PhaseEq(KernelPhase::Sealed);
        // PhaseIsTerminal subsumes PhaseEq(Sealed) because Sealed IS terminal
        assert!(terminal.subsumes(&sealed));
        // But PhaseEq(Sealed) does NOT subsume PhaseIsTerminal (Faulted is also terminal)
        assert!(!sealed.subsumes(&terminal));
    }

    #[test]
    fn processing_subsumption() {
        let processing = ConstraintExpr::PhaseIsProcessing;
        let idle = ConstraintExpr::PhaseEq(KernelPhase::Idle);
        // Idle is NOT a processing phase, so processing does NOT subsume idle
        assert!(!processing.subsumes(&idle));
    }

    #[test]
    fn cardinality_makes_sense() {
        // PhaseEq(X) covers exactly 29 cells (one phase, all inputs)
        let c = ConstraintExpr::PhaseEq(KernelPhase::Idle).cardinality();
        assert_eq!(c, KernelInputKind::ALL.len());

        // InputEq(X) covers exactly 18 cells (all phases, one input)
        let c = ConstraintExpr::InputEq(KernelInputKind::FaultDetected).cardinality();
        assert_eq!(c, KernelPhase::ALL.len());

        // True covers all 522 cells
        assert_eq!(ConstraintExpr::True.cardinality(),
            KernelPhase::ALL.len() * KernelInputKind::ALL.len());
    }

    #[test]
    fn extension_intersection() {
        let a = ConstraintExpr::PhaseEq(KernelPhase::Idle);
        let b = ConstraintExpr::InputEq(KernelInputKind::EventArrived);
        let intersection = a.and(b).cardinality();
        assert_eq!(intersection, 1); // exactly one cell: (Idle, EventArrived)
    }

    // ── Rule set consistency ─────────────────────────────────────────────

    #[test]
    fn constraint_ruleset_is_consistent() {
        let report = ruleset().verify_consistency();
        assert!(report.is_consistent,
            "constraint rule set has conflicts:\n{}",
            report.conflicts.iter().map(|c| format!(
                "  {} vs {}: {:?}", c.rule_a, c.rule_b, c.conflicting_cells
            )).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn pipeline_is_acyclic() {
        let proof = ruleset().prove_pipeline_acyclic();
        assert!(proof.is_acyclic,
            "processing pipeline has cycles: {:?}", proof.cycles);
        println!("All {} processing phases are forward-only", proof.forward_only_phases.len());
    }

    #[test]
    fn constraint_delta_matches_kernel_delta() {
        use crate::kernel_state::kernel_delta;
        let rs = ruleset();
        let cases = [
            (KernelPhase::Genesis,             KernelInputKind::Initialize,       KernelPhase::Bootstrapping),
            (KernelPhase::Idle,                KernelInputKind::EventArrived,     KernelPhase::ValidatingAbi),
            (KernelPhase::ValidatingAbi,       KernelInputKind::AbiValid,         KernelPhase::ValidatingSchema),
            (KernelPhase::ValidatingAbi,       KernelInputKind::AbiFailed,        KernelPhase::Idle),
            (KernelPhase::Deciding,            KernelInputKind::DecisionAllow,    KernelPhase::Applying),
            (KernelPhase::Applying,            KernelInputKind::TransitionFailed, KernelPhase::Recovering),
            (KernelPhase::Emitting,            KernelInputKind::EmitComplete,     KernelPhase::Idle),
            (KernelPhase::Sealed,              KernelInputKind::EventArrived,     KernelPhase::Sealed),
            (KernelPhase::Faulted,             KernelInputKind::RecoveryAttempted,KernelPhase::Recovering),
            (KernelPhase::VerifyingIdentity,   KernelInputKind::IdentityDiverged, KernelPhase::Faulted),
        ];
        for (from, input, expected) in cases {
            let from_constraints = rs.delta(from, input);
            let from_kernel = kernel_delta(from, input);
            assert_eq!(from_constraints, expected,
                "constraint δ({from}, {input:?}) = {from_constraints}, expected {expected}");
            assert_eq!(from_kernel, expected,
                "kernel δ({from}, {input:?}) = {from_kernel}, expected {expected}");
        }
    }

    #[test]
    fn fault_rule_covers_all_non_terminal_phases() {
        let rs = ruleset();
        for &phase in KernelPhase::ALL {
            if !phase.is_terminal() && phase != KernelPhase::Genesis {
                let target = rs.delta(phase, KernelInputKind::FaultDetected);
                assert_eq!(target, KernelPhase::Faulted,
                    "universal-fault rule must cover {phase}");
            }
        }
    }

    #[test]
    fn delta_is_total_via_constraints() {
        let rs = ruleset();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let _ = rs.delta(phase, input); // must not panic
            }
        }
    }

    #[test]
    fn rule_count_less_than_table_entries() {
        let rs = ruleset();
        let explicit_count = crate::kernel_state::TRANSITION_TABLE.len();
        assert!(rs.rule_count() < explicit_count,
            "constraint rules ({}) should be fewer than explicit entries ({})",
            rs.rule_count(), explicit_count);
    }

    #[test]
    fn subsumption_detected() {
        // The universal-fault rule subsumes any single-phase FaultDetected rule
        let universal = ConstraintRule::new(
            "universal-fault",
            ConstraintExpr::PhaseIsTerminal.negate()
                .and(ConstraintExpr::InputEq(KernelInputKind::FaultDetected)),
            ConstraintTarget::Phase(KernelPhase::Faulted),
        );
        let specific = ConstraintRule::new(
            "specific-fault",
            ConstraintExpr::PhaseEq(KernelPhase::Idle)
                .and(ConstraintExpr::InputEq(KernelInputKind::FaultDetected)),
            ConstraintTarget::Phase(KernelPhase::Faulted),
        );
        assert!(universal.subsumes_guard_of(&specific),
            "universal-fault must subsume specific-fault");
        assert!(!specific.subsumes_guard_of(&universal),
            "specific-fault must NOT subsume universal-fault");
    }
}
