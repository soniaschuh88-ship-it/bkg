pub mod arbitrator; pub mod pipeline; pub mod constraint_algebra;
pub mod proof_certificate;
pub mod trace_synthesizer;
pub mod specification_drift;
pub mod specification_entropy;
pub mod algebra_stability;
pub mod semantic_weight;
pub mod rule_simplifier;
pub mod counterfactual;
pub mod integration;
pub mod realm;
pub mod effect_contract;
pub mod rule_engine;
pub mod event_ledger; pub mod kernel_machine; pub mod kernel_state; pub mod state_transition; pub mod genesis; pub mod router; pub mod validator;
pub use kernel_state::{KernelPhase, KernelInputKind, KernelEffect, TransitionEntry, TRANSITION_TABLE, TransitionTableStats, kernel_delta, kernel_effects};
pub use kernel_machine::{KernelMachine, KernelContext, KernelFault, KernelFaultKind, TransitionRecord};
pub use state_transition::{ReplayIdentityProof, ReplayIdentityVerifier, ReplaySession, StateTransition, StateTransitionFn, TransitionLog};
pub use pipeline::{EventPipeline, KernelDecision, PipelineConfig, PipelineEvent, PipelineResult, PipelineStage, RejectionReason};
pub use arbitrator::{ArbitrationDecision, ArbitrationError, KernelArbitrator};
pub use genesis::Genesis;
pub use router::RealmRouter;
pub use validator::CausalContractValidator;

pub use constraint_algebra::{ConstraintExpr, ConstraintRule, ConstraintTarget, RuleSet, canonical_constraint_rules};
pub use proof_certificate::{CertificateBuilder, ExecutionTrace, ProofCheckResult, ProofChecker, TransitionCertificate};
pub use semantic_weight::{SemanticWeightLayer, SemanticWeightReport, RuleSemanticWeight, RuleNecessityProof, NecessityClass, RuleRecommendation};

pub use rule_simplifier::{RuleSimplifier, SimplificationResult, SimplificationOp, verify_canonical_is_minimal};

pub use counterfactual::{CounterfactualAnalyzer, CounterfactualAnalysis, CounterfactualReport, CounterfactualWitness, StateReachabilityGraph, SemanticFixationGuard, SimplificationVerdict, PreserveReason};
