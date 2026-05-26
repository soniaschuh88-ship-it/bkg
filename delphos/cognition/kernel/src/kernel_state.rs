// kernel_state.rs — Formal state machine definition.
//
// M = (Q, Σ, Λ, δ, λ, q₀) — a Mealy machine.
//
//   Q  = KernelPhase         finite, exhaustively enumerated
//   Σ  = KernelInputKind     finite, exhaustively enumerated
//   Λ  = KernelEffect        observable side-effects
//   δ  = kernel_delta        Q × Σ → Q, TOTAL (every cell defined)
//   λ  = kernel_effects      Q × Σ → Vec<KernelEffect>
//   q₀ = KernelPhase::Genesis
//
// Properties that MUST hold (structurally enforced):
//
//   1. TOTAL:        δ is defined for every (Q, Σ) pair.
//                    Undefined transitions → KernelPhase::Faulted.
//   2. DETERMINISTIC:δ(q, σ) returns the same q' for the same inputs, always.
//   3. CONVERGENT:   From Faulted, only RecoveryAttempted → Recovering is valid.
//                    From Sealed,  all inputs → Sealed.
//   4. MONOTONE:     The processing pipeline phases are totally ordered.
//                    No phase may transition to an earlier processing phase
//                    except via the explicit reset arcs (Idle ← Emitting).
//
// This file contains ONLY the formal definition.
// The runner (KernelMachine) lives in kernel_machine.rs.
//
// Single source of truth for DELPHOS kernel semantics.

use serde::{Deserialize, Serialize};

// ─── Q: The state space ───────────────────────────────────────────────────────

/// The complete set of kernel phases.
/// Each phase is a distinct, named position in the kernel's lifecycle.
/// No hidden states. No implicit state embedded in fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KernelPhase {
    // ── Lifecycle ──────────────────────────────────────────────────────────
    /// Kernel created. No events ever processed. Genesis hash not yet set.
    Genesis,
    /// Genesis event is being committed. Initial state is being built.
    Bootstrapping,
    /// Ready to accept events. No event currently in flight.
    Idle,

    // ── Processing pipeline (strictly ordered, no backward transitions) ───
    /// ABI envelope version check in progress.
    ValidatingAbi,
    /// EventSchemaRegistry lookup in progress.
    ValidatingSchema,
    /// Lamport clock monotone check in progress.
    ValidatingClock,
    /// Capability grant check in progress.
    ValidatingCapability,
    /// Causal parent existence check in progress.
    ValidatingCausal,
    /// Computing KernelDecision (Allow | Reject | Transform).
    Deciding,
    /// Running Reducer<E>: applying the state transition.
    Applying,
    /// MaterializerKernel.stamp(): issuing KernelStamp.
    Stamping,
    /// Appending to ledger + broadcasting to subscribers.
    Emitting,

    // ── Replay ────────────────────────────────────────────────────────────
    /// Replay requested; waiting for ledger read to begin.
    ReplayPending,
    /// Applying historical events in sequence.
    Replaying,
    /// ReplayIdentityVerifier.verify() running.
    VerifyingIdentity,

    // ── Recovery ──────────────────────────────────────────────────────────
    /// A recoverable fault; attempting automatic recovery.
    Recovering,

    // ── Terminal (no exit without explicit operator action) ────────────────
    /// Archive mode. No more events accepted.
    Sealed,
    /// Unrecoverable fault. Operator intervention required.
    Faulted,
}

impl KernelPhase {
    /// All phases in definition order. Used for exhaustiveness checks.
    pub const ALL: &'static [KernelPhase] = &[
        Self::Genesis, Self::Bootstrapping, Self::Idle,
        Self::ValidatingAbi, Self::ValidatingSchema, Self::ValidatingClock,
        Self::ValidatingCapability, Self::ValidatingCausal, Self::Deciding,
        Self::Applying, Self::Stamping, Self::Emitting,
        Self::ReplayPending, Self::Replaying, Self::VerifyingIdentity,
        Self::Recovering, Self::Sealed, Self::Faulted,
    ];

    pub fn is_processing(self) -> bool {
        matches!(self,
            Self::ValidatingAbi | Self::ValidatingSchema | Self::ValidatingClock
            | Self::ValidatingCapability | Self::ValidatingCausal | Self::Deciding
            | Self::Applying | Self::Stamping | Self::Emitting
        )
    }

    pub fn is_terminal(self) -> bool { matches!(self, Self::Sealed | Self::Faulted) }
    pub fn is_replaying(self) -> bool { matches!(self, Self::ReplayPending | Self::Replaying | Self::VerifyingIdentity) }
    pub fn is_healthy(self) -> bool { !matches!(self, Self::Faulted) }

    /// The expected successor in the normal processing pipeline.
    /// Returns None for phases outside the processing pipeline.
    pub fn next_processing_phase(self) -> Option<KernelPhase> {
        match self {
            Self::Idle              => Some(Self::ValidatingAbi),
            Self::ValidatingAbi     => Some(Self::ValidatingSchema),
            Self::ValidatingSchema  => Some(Self::ValidatingClock),
            Self::ValidatingClock   => Some(Self::ValidatingCapability),
            Self::ValidatingCapability => Some(Self::ValidatingCausal),
            Self::ValidatingCausal  => Some(Self::Deciding),
            Self::Deciding          => Some(Self::Applying),
            Self::Applying          => Some(Self::Stamping),
            Self::Stamping          => Some(Self::Emitting),
            Self::Emitting          => Some(Self::Idle), // cycle back
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Genesis            => "Genesis",
            Self::Bootstrapping      => "Bootstrapping",
            Self::Idle               => "Idle",
            Self::ValidatingAbi      => "ValidatingAbi",
            Self::ValidatingSchema   => "ValidatingSchema",
            Self::ValidatingClock    => "ValidatingClock",
            Self::ValidatingCapability => "ValidatingCapability",
            Self::ValidatingCausal   => "ValidatingCausal",
            Self::Deciding           => "Deciding",
            Self::Applying           => "Applying",
            Self::Stamping           => "Stamping",
            Self::Emitting           => "Emitting",
            Self::ReplayPending      => "ReplayPending",
            Self::Replaying          => "Replaying",
            Self::VerifyingIdentity  => "VerifyingIdentity",
            Self::Recovering         => "Recovering",
            Self::Sealed             => "Sealed",
            Self::Faulted            => "Faulted",
        }
    }
}

impl std::fmt::Display for KernelPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) }
}

// ─── Σ: The input alphabet ────────────────────────────────────────────────────

/// The complete input alphabet — every stimulus the kernel can receive.
/// Used as the key in the transition table δ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KernelInputKind {
    // ── Lifecycle ────────────────────────────────────────────────────────
    /// Operator initializes the kernel with a genesis hash.
    Initialize,
    /// Bootstrap complete — genesis state committed.
    BootstrapComplete,

    // ── Event processing ─────────────────────────────────────────────────
    /// New event arrived from the bus.
    EventArrived,
    /// ABI validation passed.
    AbiValid,
    /// ABI validation failed.
    AbiFailed,
    /// Schema validation passed.
    SchemaValid,
    /// Schema validation failed.
    SchemaFailed,
    /// Clock validation passed.
    ClockValid,
    /// Clock validation failed.
    ClockFailed,
    /// Capability check passed.
    CapabilityGranted,
    /// Capability check failed.
    CapabilityDenied,
    /// Causal parent validation passed.
    CausalValid,
    /// Causal parent validation failed.
    CausalFailed,
    /// Decision: Allow.
    DecisionAllow,
    /// Decision: Reject.
    DecisionReject,
    /// Decision: Transform.
    DecisionTransform,
    /// State transition applied successfully.
    TransitionApplied,
    /// State transition failed.
    TransitionFailed,
    /// Projection stamped by MaterializerKernel.
    ProjectionStamped,
    /// Event emitted to ledger + subscribers.
    EmitComplete,

    // ── Replay ───────────────────────────────────────────────────────────
    /// Replay requested by operator or recovery subsystem.
    ReplayRequested,
    /// One event applied during replay.
    ReplayEventApplied,
    /// Replay complete — all events applied.
    ReplayComplete,
    /// ReplayIdentityVerifier confirmed identity.
    IdentityConfirmed,
    /// ReplayIdentityVerifier detected divergence.
    IdentityDiverged,

    // ── Lifecycle control ────────────────────────────────────────────────
    /// Operator seals the kernel.
    SealRequested,
    /// Unrecoverable fault detected.
    FaultDetected,
    /// Automatic recovery attempted.
    RecoveryAttempted,
    /// Recovery succeeded.
    RecoverySucceeded,
}

impl KernelInputKind {
    pub const ALL: &'static [KernelInputKind] = &[
        Self::Initialize, Self::BootstrapComplete,
        Self::EventArrived,
        Self::AbiValid, Self::AbiFailed,
        Self::SchemaValid, Self::SchemaFailed,
        Self::ClockValid, Self::ClockFailed,
        Self::CapabilityGranted, Self::CapabilityDenied,
        Self::CausalValid, Self::CausalFailed,
        Self::DecisionAllow, Self::DecisionReject, Self::DecisionTransform,
        Self::TransitionApplied, Self::TransitionFailed,
        Self::ProjectionStamped, Self::EmitComplete,
        Self::ReplayRequested, Self::ReplayEventApplied,
        Self::ReplayComplete, Self::IdentityConfirmed, Self::IdentityDiverged,
        Self::SealRequested, Self::FaultDetected,
        Self::RecoveryAttempted, Self::RecoverySucceeded,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Initialize        => "Initialize",
            Self::BootstrapComplete => "BootstrapComplete",
            Self::EventArrived      => "EventArrived",
            Self::AbiValid          => "AbiValid",
            Self::AbiFailed         => "AbiFailed",
            Self::SchemaValid       => "SchemaValid",
            Self::SchemaFailed      => "SchemaFailed",
            Self::ClockValid        => "ClockValid",
            Self::ClockFailed       => "ClockFailed",
            Self::CapabilityGranted => "CapabilityGranted",
            Self::CapabilityDenied  => "CapabilityDenied",
            Self::CausalValid       => "CausalValid",
            Self::CausalFailed      => "CausalFailed",
            Self::DecisionAllow     => "DecisionAllow",
            Self::DecisionReject    => "DecisionReject",
            Self::DecisionTransform => "DecisionTransform",
            Self::TransitionApplied => "TransitionApplied",
            Self::TransitionFailed  => "TransitionFailed",
            Self::ProjectionStamped => "ProjectionStamped",
            Self::EmitComplete      => "EmitComplete",
            Self::ReplayRequested   => "ReplayRequested",
            Self::ReplayEventApplied=> "ReplayEventApplied",
            Self::ReplayComplete    => "ReplayComplete",
            Self::IdentityConfirmed => "IdentityConfirmed",
            Self::IdentityDiverged  => "IdentityDiverged",
            Self::SealRequested     => "SealRequested",
            Self::FaultDetected     => "FaultDetected",
            Self::RecoveryAttempted => "RecoveryAttempted",
            Self::RecoverySucceeded => "RecoverySucceeded",
        }
    }
}

// ─── Λ: Observable effects ────────────────────────────────────────────────────

/// Observable side-effects emitted by transitions.
/// These are what the outside world sees — not state changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEffect {
    /// Kernel is now ready to accept events.
    KernelReady,
    /// An event was rejected — carry reason string.
    EventRejected(String),
    /// An event was accepted and applied.
    EventAccepted { event_id: String },
    /// A projection was stamped and is now valid.
    ProjectionValid { projection_id: String },
    /// Replay started.
    ReplayStarted,
    /// Replay identity confirmed — system is deterministic.
    IdentityConfirmed { transition_count: u64 },
    /// Replay identity diverged — DETERMINISM FAILURE.
    DeterminismFailure { reason: String },
    /// Kernel sealed — no more events.
    Sealed,
    /// Fault recorded.
    FaultRecorded(String),
    /// Recovery complete.
    Recovered,
}

// ─── δ: The total transition function ────────────────────────────────────────

/// A single entry in the transition table.
#[derive(Debug, Clone, Copy)]
pub struct TransitionEntry {
    pub from: KernelPhase,
    pub on: KernelInputKind,
    pub to: KernelPhase,
}

impl TransitionEntry {
    const fn new(from: KernelPhase, on: KernelInputKind, to: KernelPhase) -> Self {
        Self { from, on, to }
    }
}

/// The explicit transition table — the complete definition of δ.
/// Every valid transition is listed here exactly once.
/// Any (phase, input) NOT in this table → KernelPhase::Faulted.
/// This makes δ TOTAL.
pub static TRANSITION_TABLE: &[TransitionEntry] = &[
    // ── Lifecycle arcs ───────────────────────────────────────────────────
    TransitionEntry::new(KernelPhase::Genesis,       KernelInputKind::Initialize,        KernelPhase::Bootstrapping),
    TransitionEntry::new(KernelPhase::Bootstrapping, KernelInputKind::BootstrapComplete, KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::Bootstrapping, KernelInputKind::FaultDetected,     KernelPhase::Faulted),

    // ── Main processing pipeline ─────────────────────────────────────────
    TransitionEntry::new(KernelPhase::Idle,                  KernelInputKind::EventArrived,       KernelPhase::ValidatingAbi),
    TransitionEntry::new(KernelPhase::ValidatingAbi,         KernelInputKind::AbiValid,           KernelPhase::ValidatingSchema),
    TransitionEntry::new(KernelPhase::ValidatingAbi,         KernelInputKind::AbiFailed,          KernelPhase::Idle),     // reject → back to Idle
    TransitionEntry::new(KernelPhase::ValidatingSchema,      KernelInputKind::SchemaValid,        KernelPhase::ValidatingClock),
    TransitionEntry::new(KernelPhase::ValidatingSchema,      KernelInputKind::SchemaFailed,       KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::ValidatingClock,       KernelInputKind::ClockValid,         KernelPhase::ValidatingCapability),
    TransitionEntry::new(KernelPhase::ValidatingClock,       KernelInputKind::ClockFailed,        KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::ValidatingCapability,  KernelInputKind::CapabilityGranted,  KernelPhase::ValidatingCausal),
    TransitionEntry::new(KernelPhase::ValidatingCapability,  KernelInputKind::CapabilityDenied,   KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::ValidatingCausal,      KernelInputKind::CausalValid,        KernelPhase::Deciding),
    TransitionEntry::new(KernelPhase::ValidatingCausal,      KernelInputKind::CausalFailed,       KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::Deciding,              KernelInputKind::DecisionAllow,      KernelPhase::Applying),
    TransitionEntry::new(KernelPhase::Deciding,              KernelInputKind::DecisionReject,     KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::Deciding,              KernelInputKind::DecisionTransform,  KernelPhase::Applying),
    TransitionEntry::new(KernelPhase::Applying,              KernelInputKind::TransitionApplied,  KernelPhase::Stamping),
    TransitionEntry::new(KernelPhase::Applying,              KernelInputKind::TransitionFailed,   KernelPhase::Recovering),
    TransitionEntry::new(KernelPhase::Stamping,              KernelInputKind::ProjectionStamped,  KernelPhase::Emitting),
    TransitionEntry::new(KernelPhase::Emitting,              KernelInputKind::EmitComplete,       KernelPhase::Idle),     // normal cycle

    // ── Replay arcs ──────────────────────────────────────────────────────
    TransitionEntry::new(KernelPhase::Idle,             KernelInputKind::ReplayRequested,     KernelPhase::ReplayPending),
    TransitionEntry::new(KernelPhase::ReplayPending,    KernelInputKind::EventArrived,        KernelPhase::Replaying),
    TransitionEntry::new(KernelPhase::Replaying,        KernelInputKind::ReplayEventApplied,  KernelPhase::Replaying),   // self-loop
    TransitionEntry::new(KernelPhase::Replaying,        KernelInputKind::ReplayComplete,      KernelPhase::VerifyingIdentity),
    TransitionEntry::new(KernelPhase::VerifyingIdentity,KernelInputKind::IdentityConfirmed,   KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::VerifyingIdentity,KernelInputKind::IdentityDiverged,    KernelPhase::Faulted),     // determinism failure

    // ── Recovery arcs ────────────────────────────────────────────────────
    TransitionEntry::new(KernelPhase::Recovering,      KernelInputKind::RecoveryAttempted,   KernelPhase::Recovering),  // still trying
    TransitionEntry::new(KernelPhase::Recovering,      KernelInputKind::RecoverySucceeded,   KernelPhase::Idle),
    TransitionEntry::new(KernelPhase::Recovering,      KernelInputKind::FaultDetected,       KernelPhase::Faulted),

    // ── Universal arcs: FaultDetected from any non-terminal phase → Faulted
    TransitionEntry::new(KernelPhase::Idle,                  KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::ValidatingAbi,         KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::ValidatingSchema,      KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::ValidatingClock,       KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::ValidatingCapability,  KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::ValidatingCausal,      KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::Deciding,              KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::Applying,              KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::Stamping,              KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::Emitting,              KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::ReplayPending,         KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::Replaying,             KernelInputKind::FaultDetected,  KernelPhase::Faulted),
    TransitionEntry::new(KernelPhase::VerifyingIdentity,     KernelInputKind::FaultDetected,  KernelPhase::Faulted),

    // ── Seal arcs: SealRequested from Idle → Sealed ──────────────────────
    TransitionEntry::new(KernelPhase::Idle,     KernelInputKind::SealRequested,  KernelPhase::Sealed),
    TransitionEntry::new(KernelPhase::Replaying,KernelInputKind::SealRequested,  KernelPhase::Sealed),
];

// ─── δ: kernel_delta — the total transition function ─────────────────────────

/// δ(q, σ) → q' — the total transition function.
///
/// TOTAL: every (q, σ) pair has an explicit result.
/// Pairs not in TRANSITION_TABLE → KernelPhase::Faulted.
/// Sealed absorbs all inputs (absorbing state).
/// Faulted absorbs all inputs except RecoveryAttempted.
pub fn kernel_delta(phase: KernelPhase, input: KernelInputKind) -> KernelPhase {
    // Absorbing states first (avoids table lookup)
    if phase == KernelPhase::Sealed {
        return KernelPhase::Sealed;
    }
    if phase == KernelPhase::Faulted {
        return match input {
            KernelInputKind::RecoveryAttempted => KernelPhase::Recovering,
            _ => KernelPhase::Faulted, // absorbing
        };
    }

    // Table lookup
    for entry in TRANSITION_TABLE {
        if entry.from == phase && entry.on == input {
            return entry.to;
        }
    }

    // Not in table → undefined transition → Faulted (makes δ total)
    KernelPhase::Faulted
}

/// λ(q, σ) → Vec<KernelEffect> — the output function.
/// Produces observable effects for a given (phase, input) transition.
pub fn kernel_effects(phase: KernelPhase, input: KernelInputKind) -> Vec<KernelEffect> {
    match (phase, input) {
        (KernelPhase::Bootstrapping, KernelInputKind::BootstrapComplete) =>
            vec![KernelEffect::KernelReady],
        (KernelPhase::ValidatingAbi, KernelInputKind::AbiFailed)
        | (KernelPhase::ValidatingSchema, KernelInputKind::SchemaFailed)
        | (KernelPhase::ValidatingClock, KernelInputKind::ClockFailed)
        | (KernelPhase::ValidatingCapability, KernelInputKind::CapabilityDenied)
        | (KernelPhase::ValidatingCausal, KernelInputKind::CausalFailed)
        | (KernelPhase::Deciding, KernelInputKind::DecisionReject) =>
            vec![KernelEffect::EventRejected(format!("rejected at {phase}"))],
        (KernelPhase::Emitting, KernelInputKind::EmitComplete) =>
            vec![KernelEffect::EventAccepted { event_id: String::new() }],
        (KernelPhase::VerifyingIdentity, KernelInputKind::IdentityConfirmed) =>
            vec![KernelEffect::IdentityConfirmed { transition_count: 0 }],
        (KernelPhase::VerifyingIdentity, KernelInputKind::IdentityDiverged) =>
            vec![KernelEffect::DeterminismFailure { reason: "checksum mismatch".into() }],
        (KernelPhase::Idle, KernelInputKind::SealRequested) =>
            vec![KernelEffect::Sealed],
        (_, KernelInputKind::FaultDetected) =>
            vec![KernelEffect::FaultRecorded(format!("fault at {phase}"))],
        (KernelPhase::Recovering, KernelInputKind::RecoverySucceeded) =>
            vec![KernelEffect::Recovered],
        _ => vec![],
    }
}

// ─── Transition table analysis ────────────────────────────────────────────────

/// Metadata about the transition table — used for formal verification.
pub struct TransitionTableStats {
    pub phase_count: usize,
    pub input_count: usize,
    pub defined_transitions: usize,
    pub undefined_transitions: usize,  // → Faulted by default
    pub self_loops: usize,
}

impl TransitionTableStats {
    pub fn compute() -> Self {
        let phase_count = KernelPhase::ALL.len();
        let input_count = KernelInputKind::ALL.len();
        let total = phase_count * input_count;
        let defined = TRANSITION_TABLE.len();
        // Absorbing states absorb all remaining inputs
        let absorbing_states = 2; // Sealed + Faulted
        let absorbing_inputs = absorbing_states * input_count;
        let self_loops = TRANSITION_TABLE.iter().filter(|e| e.from == e.to).count();
        Self {
            phase_count,
            input_count,
            defined_transitions: defined + absorbing_inputs,
            undefined_transitions: total.saturating_sub(defined + absorbing_inputs),
            self_loops,
        }
    }

    pub fn coverage_percent(&self) -> f64 {
        let total = self.phase_count * self.input_count;
        100.0 * self.defined_transitions as f64 / total as f64
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Property 1: δ is total ─────────────────────────────────────────────

    #[test]
    fn delta_is_total_never_panics() {
        // δ(q, σ) must return SOMETHING for every (q, σ) pair.
        // If it panics → it's a partial function → test fails.
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let result = kernel_delta(phase, input);
                // Just calling it is the test — it must not panic
                let _ = result;
            }
        }
    }

    #[test]
    fn undefined_transitions_go_to_faulted() {
        // An input with no defined transition from a given phase → Faulted
        // Genesis + EventArrived is not in the table → Faulted
        assert_eq!(kernel_delta(KernelPhase::Genesis, KernelInputKind::EventArrived), KernelPhase::Faulted);
        // Idle + AbiValid (not in table — no event in flight) → Faulted
        assert_eq!(kernel_delta(KernelPhase::Idle, KernelInputKind::AbiValid), KernelPhase::Faulted);
    }

    // ── Property 2: δ is deterministic ────────────────────────────────────

    #[test]
    fn delta_is_deterministic() {
        // Same inputs → same output, always. Test 1000 random calls.
        let cases = [
            (KernelPhase::Genesis, KernelInputKind::Initialize, KernelPhase::Bootstrapping),
            (KernelPhase::Idle, KernelInputKind::EventArrived, KernelPhase::ValidatingAbi),
            (KernelPhase::Faulted, KernelInputKind::EventArrived, KernelPhase::Faulted),
            (KernelPhase::Sealed, KernelInputKind::EventArrived, KernelPhase::Sealed),
        ];
        for (from, input, expected) in cases {
            for _ in 0..100 { // call 100 times — must always return same result
                assert_eq!(kernel_delta(from, input), expected);
            }
        }
    }

    // ── Property 3: Absorbing states ──────────────────────────────────────

    #[test]
    fn sealed_absorbs_all_inputs() {
        for &input in KernelInputKind::ALL {
            assert_eq!(kernel_delta(KernelPhase::Sealed, input), KernelPhase::Sealed,
                "Sealed must absorb {input:?}");
        }
    }

    #[test]
    fn faulted_absorbs_non_recovery_inputs() {
        for &input in KernelInputKind::ALL {
            let result = kernel_delta(KernelPhase::Faulted, input);
            if input == KernelInputKind::RecoveryAttempted {
                assert_eq!(result, KernelPhase::Recovering);
            } else {
                assert_eq!(result, KernelPhase::Faulted,
                    "Faulted must absorb {input:?}");
            }
        }
    }

    // ── Property 4: Processing pipeline is totally ordered ────────────────

    #[test]
    fn processing_pipeline_order() {
        // Follow the happy path through the processing pipeline
        let happy_path = [
            (KernelPhase::Idle,                 KernelInputKind::EventArrived,       KernelPhase::ValidatingAbi),
            (KernelPhase::ValidatingAbi,         KernelInputKind::AbiValid,           KernelPhase::ValidatingSchema),
            (KernelPhase::ValidatingSchema,      KernelInputKind::SchemaValid,        KernelPhase::ValidatingClock),
            (KernelPhase::ValidatingClock,       KernelInputKind::ClockValid,         KernelPhase::ValidatingCapability),
            (KernelPhase::ValidatingCapability,  KernelInputKind::CapabilityGranted,  KernelPhase::ValidatingCausal),
            (KernelPhase::ValidatingCausal,      KernelInputKind::CausalValid,        KernelPhase::Deciding),
            (KernelPhase::Deciding,              KernelInputKind::DecisionAllow,      KernelPhase::Applying),
            (KernelPhase::Applying,              KernelInputKind::TransitionApplied,  KernelPhase::Stamping),
            (KernelPhase::Stamping,              KernelInputKind::ProjectionStamped,  KernelPhase::Emitting),
            (KernelPhase::Emitting,              KernelInputKind::EmitComplete,       KernelPhase::Idle),
        ];
        for (from, input, expected) in happy_path {
            assert_eq!(kernel_delta(from, input), expected,
                "pipeline step {from} --{input:?}--> should be {expected}");
        }
    }

    #[test]
    fn processing_pipeline_cycles_back_to_idle() {
        // Emitting → EmitComplete → Idle (not a terminal state)
        assert_eq!(kernel_delta(KernelPhase::Emitting, KernelInputKind::EmitComplete), KernelPhase::Idle);
        assert!(!KernelPhase::Idle.is_terminal());
    }

    // ── Property 5: Fault from any processing phase ────────────────────────

    #[test]
    fn fault_from_any_processing_phase() {
        for &phase in KernelPhase::ALL {
            if !phase.is_terminal() && phase != KernelPhase::Genesis {
                let result = kernel_delta(phase, KernelInputKind::FaultDetected);
                assert_eq!(result, KernelPhase::Faulted,
                    "FaultDetected from {phase} must → Faulted");
            }
        }
    }

    // ── Property 6: Replay arc ────────────────────────────────────────────

    #[test]
    fn replay_arc_complete() {
        assert_eq!(kernel_delta(KernelPhase::Idle, KernelInputKind::ReplayRequested), KernelPhase::ReplayPending);
        assert_eq!(kernel_delta(KernelPhase::ReplayPending, KernelInputKind::EventArrived), KernelPhase::Replaying);
        assert_eq!(kernel_delta(KernelPhase::Replaying, KernelInputKind::ReplayEventApplied), KernelPhase::Replaying); // self-loop
        assert_eq!(kernel_delta(KernelPhase::Replaying, KernelInputKind::ReplayComplete), KernelPhase::VerifyingIdentity);
        assert_eq!(kernel_delta(KernelPhase::VerifyingIdentity, KernelInputKind::IdentityConfirmed), KernelPhase::Idle);
        // Divergence → Faulted (determinism failure is terminal)
        assert_eq!(kernel_delta(KernelPhase::VerifyingIdentity, KernelInputKind::IdentityDiverged), KernelPhase::Faulted);
    }

    // ── Property 7: Recovery arc ──────────────────────────────────────────

    #[test]
    fn recovery_arc() {
        assert_eq!(kernel_delta(KernelPhase::Applying, KernelInputKind::TransitionFailed), KernelPhase::Recovering);
        assert_eq!(kernel_delta(KernelPhase::Recovering, KernelInputKind::RecoverySucceeded), KernelPhase::Idle);
        assert_eq!(kernel_delta(KernelPhase::Recovering, KernelInputKind::FaultDetected), KernelPhase::Faulted);
    }

    // ── Property 8: next_processing_phase is consistent with table ─────────

    #[test]
    fn next_processing_phase_consistent() {
        // Each phase + its "advance" input → next phase
        // next_processing_phase() must agree with kernel_delta
        let steps: &[(KernelPhase, KernelInputKind)] = &[
            (KernelPhase::Idle,                 KernelInputKind::EventArrived),
            (KernelPhase::ValidatingAbi,         KernelInputKind::AbiValid),
            (KernelPhase::ValidatingSchema,      KernelInputKind::SchemaValid),
            (KernelPhase::ValidatingClock,       KernelInputKind::ClockValid),
            (KernelPhase::ValidatingCapability,  KernelInputKind::CapabilityGranted),
            (KernelPhase::ValidatingCausal,      KernelInputKind::CausalValid),
            (KernelPhase::Deciding,              KernelInputKind::DecisionAllow),
            (KernelPhase::Applying,              KernelInputKind::TransitionApplied),
            (KernelPhase::Stamping,              KernelInputKind::ProjectionStamped),
            (KernelPhase::Emitting,              KernelInputKind::EmitComplete),
        ];
        for &(phase, input) in steps {
            let from_table = kernel_delta(phase, input);
            let from_helper = phase.next_processing_phase().unwrap();
            assert_eq!(from_table, from_helper,
                "table and next_processing_phase must agree for {phase}");
        }
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    #[test]
    fn transition_table_stats() {
        let stats = TransitionTableStats::compute();
        assert_eq!(stats.phase_count, KernelPhase::ALL.len());
        assert_eq!(stats.input_count, KernelInputKind::ALL.len());
        // Coverage should be reasonable — we define many transitions
        // Coverage includes absorbing states (Sealed/Faulted absorb all inputs)
        println!("Transition table: {} phases × {} inputs = {} total, {} defined ({:.1}%)",
            stats.phase_count, stats.input_count, stats.phase_count * stats.input_count,
            stats.defined_transitions, stats.coverage_percent());
        assert!(stats.defined_transitions > 0);
    }

    #[test]
    fn no_duplicate_table_entries() {
        let mut seen = std::collections::HashSet::new();
        for entry in TRANSITION_TABLE {
            let key = (entry.from, entry.on);
            assert!(seen.insert(key),
                "duplicate entry for ({}, {:?})", entry.from, entry.on);
        }
    }
}
