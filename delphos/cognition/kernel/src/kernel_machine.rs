// kernel_machine.rs — The KernelMachine: the runner for the formal state machine.
//
// KernelMachine wraps the formal (Q, Σ, Λ, δ, λ, q₀) definition from
// kernel_state.rs and adds:
//
//   - KernelContext: the data that travels with transitions
//   - TransitionRecord: an immutable record of every δ invocation
//   - step(): the public API for advancing the machine
//   - A complete history log for replay and audit
//
// Properties preserved by KernelMachine:
//
//   1. TOTAL:        step() never panics — undefined δ returns Faulted
//   2. DETERMINISTIC:step(input) with same context → same new phase
//   3. APPEND-ONLY:  history is never modified after recording
//   4. OBSERVABLE:   every step produces effects (possibly empty)
//
// Single source of truth for kernel execution.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_core::RealmId;

use crate::kernel_state::{
    kernel_delta, kernel_effects,
    KernelEffect, KernelInputKind, KernelPhase,
};

// ─── KernelFault ─────────────────────────────────────────────────────────────

/// A classified, immutable fault record.
/// Faults are permanent — they require operator review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelFault {
    pub id: String,
    pub kind: KernelFaultKind,
    pub phase_at_fault: KernelPhase,
    pub input_at_fault: KernelInputKind,
    pub message: String,
    pub realm_id: Option<RealmId>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelFaultKind {
    /// An invalid transition was attempted (not in transition table).
    InvalidTransition,
    /// Replay identity diverged — determinism failure.
    DeterminismViolation,
    /// A state transition invariant was violated.
    InvariantViolation,
    /// External fault (hardware, network, etc.).
    ExternalFault,
}

impl KernelFault {
    fn new(kind: KernelFaultKind, phase: KernelPhase, input: KernelInputKind, msg: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind, phase_at_fault: phase, input_at_fault: input,
            message: msg.into(), realm_id: None,
            occurred_at: Utc::now(),
        }
    }
}

// ─── TransitionRecord ─────────────────────────────────────────────────────────

/// An immutable record of one δ invocation.
/// Append-only. Never modified after creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    /// Monotone sequence number within this machine's history.
    pub seq: u64,
    pub from_phase: KernelPhase,
    pub input: KernelInputKind,
    pub to_phase: KernelPhase,
    pub effects: Vec<KernelEffect>,
    pub context_lamport: u64,
    pub recorded_at: DateTime<Utc>,
}

impl TransitionRecord {
    pub fn was_invalid(&self) -> bool {
        self.from_phase != KernelPhase::Faulted
            && self.to_phase == KernelPhase::Faulted
            && self.input != KernelInputKind::FaultDetected
    }
}

// ─── KernelContext ────────────────────────────────────────────────────────────

/// The data context passed into each transition.
/// Contains the current event being processed (if any) and runtime counters.
#[derive(Debug, Clone)]
pub struct KernelContext {
    /// Current realm being processed.
    pub realm_id: RealmId,
    /// Lamport counter of the event in flight.
    pub current_lamport: u64,
    /// Event ID in flight.
    pub current_event_id: Option<String>,
    /// Schema ID in flight.
    pub current_schema_id: Option<String>,
    /// Number of events processed since last reset.
    pub processed_count: u64,
    /// Number of events rejected since last reset.
    pub rejected_count: u64,
    /// Whether a fault is being recovered.
    pub in_recovery: bool,
}

impl KernelContext {
    pub fn new(realm_id: RealmId) -> Self {
        Self {
            realm_id, current_lamport: 0,
            current_event_id: None, current_schema_id: None,
            processed_count: 0, rejected_count: 0, in_recovery: false,
        }
    }
}

// ─── KernelMachine ────────────────────────────────────────────────────────────

/// The kernel state machine runner.
///
/// Wraps the formal (Q, Σ, Λ, δ, λ, q₀) definition.
/// Every state change goes through step() — no other mutation path.
pub struct KernelMachine {
    pub phase: KernelPhase,
    pub context: KernelContext,
    /// Append-only history of all transitions.
    history: Vec<TransitionRecord>,
    /// Active faults (cleared on recovery).
    pub faults: Vec<KernelFault>,
    seq_counter: u64,
}

impl KernelMachine {
    /// Create a new kernel in the Genesis phase.
    pub fn new(realm_id: RealmId) -> Self {
        Self {
            phase: KernelPhase::Genesis,
            context: KernelContext::new(realm_id),
            history: vec![],
            faults: vec![],
            seq_counter: 0,
        }
    }

    /// Apply one input to the machine: δ(current_phase, input) → new_phase.
    ///
    /// This is the ONLY way to advance the machine.
    /// Returns the new phase and any observable effects.
    ///
    /// Never panics. Invalid transitions → phase = Faulted + fault recorded.
    pub fn step(&mut self, input: KernelInputKind) -> (KernelPhase, Vec<KernelEffect>) {
        let from = self.phase;
        let new_phase = kernel_delta(from, input);
        let mut effects = kernel_effects(from, input);

        // Record a fault whenever we transition into Faulted from a non-faulted phase
        if from != KernelPhase::Faulted && from != KernelPhase::Sealed && new_phase == KernelPhase::Faulted {
            let (kind, msg) = if input == KernelInputKind::FaultDetected {
                (KernelFaultKind::ExternalFault, format!("fault detected at phase {from}"))
            } else if input == KernelInputKind::IdentityDiverged {
                (KernelFaultKind::DeterminismViolation, format!("replay identity diverged at {from}"))
            } else {
                (KernelFaultKind::InvalidTransition, format!("undefined transition: {from} --{input:?}-->"))
            };
            let fault = KernelFault::new(kind, from, input, msg);
            effects.push(KernelEffect::FaultRecorded(fault.message.clone()));
            self.faults.push(fault);
        }

        // Append transition to history (never modify existing records)
        self.seq_counter += 1;
        self.history.push(TransitionRecord {
            seq: self.seq_counter,
            from_phase: from,
            input,
            to_phase: new_phase,
            effects: effects.clone(),
            context_lamport: self.context.current_lamport,
            recorded_at: Utc::now(),
        });

        // Update counters
        if effects.iter().any(|e| matches!(e, KernelEffect::EventAccepted { .. })) {
            self.context.processed_count += 1;
        }
        if effects.iter().any(|e| matches!(e, KernelEffect::EventRejected(_))) {
            self.context.rejected_count += 1;
        }
        if new_phase == KernelPhase::Recovering {
            self.context.in_recovery = true;
        }
        if new_phase == KernelPhase::Idle && from == KernelPhase::Recovering {
            self.context.in_recovery = false;
        }

        self.phase = new_phase;
        (new_phase, effects)
    }

    /// Run a sequence of inputs. Returns all (phase, effects) pairs.
    pub fn run(&mut self, inputs: impl IntoIterator<Item = KernelInputKind>) -> Vec<(KernelPhase, Vec<KernelEffect>)> {
        inputs.into_iter().map(|input| self.step(input)).collect()
    }

    /// Initialize the kernel (Genesis → Bootstrapping → Idle).
    pub fn initialize(&mut self) -> Result<(), KernelInitError> {
        if self.phase != KernelPhase::Genesis {
            return Err(KernelInitError::AlreadyInitialized { current: self.phase });
        }
        self.step(KernelInputKind::Initialize);
        self.step(KernelInputKind::BootstrapComplete);
        if self.phase != KernelPhase::Idle {
            return Err(KernelInitError::InitFailed { phase: self.phase });
        }
        Ok(())
    }

    /// Simulate processing one event through the full happy-path pipeline.
    /// Used in tests to verify the complete pipeline arc.
    pub fn simulate_happy_path_event(&mut self) -> Vec<(KernelPhase, Vec<KernelEffect>)> {
        self.run([
            KernelInputKind::EventArrived,
            KernelInputKind::AbiValid,
            KernelInputKind::SchemaValid,
            KernelInputKind::ClockValid,
            KernelInputKind::CapabilityGranted,
            KernelInputKind::CausalValid,
            KernelInputKind::DecisionAllow,
            KernelInputKind::TransitionApplied,
            KernelInputKind::ProjectionStamped,
            KernelInputKind::EmitComplete,
        ])
    }

    // ── Introspection ─────────────────────────────────────────────────────

    pub fn history(&self) -> &[TransitionRecord] { &self.history }
    pub fn history_len(&self) -> usize { self.history.len() }
    pub fn is_healthy(&self) -> bool { self.phase.is_healthy() }
    pub fn is_sealed(&self) -> bool { self.phase == KernelPhase::Sealed }
    pub fn is_faulted(&self) -> bool { self.phase == KernelPhase::Faulted }
    pub fn fault_count(&self) -> usize { self.faults.len() }
    pub fn processed_count(&self) -> u64 { self.context.processed_count }
    pub fn rejected_count(&self) -> u64 { self.context.rejected_count }

    /// Phases visited (in order, deduplicated).
    pub fn phases_visited(&self) -> Vec<KernelPhase> {
        let mut v: Vec<KernelPhase> = self.history.iter().map(|r| r.from_phase).collect();
        v.push(self.phase);
        v.dedup();
        v
    }

    /// Invalid transitions in history (transitions that → Faulted unexpectedly).
    pub fn invalid_transitions(&self) -> Vec<&TransitionRecord> {
        self.history.iter().filter(|r| r.was_invalid()).collect()
    }
}

// ─── KernelInitError ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, thiserror::Error)]
pub enum KernelInitError {
    #[error("kernel already initialized (current phase: {current})")]
    AlreadyInitialized { current: KernelPhase },
    #[error("kernel init failed (stuck in: {phase})")]
    InitFailed { phase: KernelPhase },
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_state::{KernelInputKind, TransitionTableStats};
    use bkg_core::RealmId;

    fn initialized_machine() -> KernelMachine {
        let mut m = KernelMachine::new(RealmId::Telum);
        m.initialize().unwrap();
        m
    }

    // ── Test 1: Initialization arc ────────────────────────────────────────

    #[test]
    fn initialize_reaches_idle() {
        let mut m = KernelMachine::new(RealmId::Telum);
        assert_eq!(m.phase, KernelPhase::Genesis);
        m.initialize().unwrap();
        assert_eq!(m.phase, KernelPhase::Idle);
        assert!(m.is_healthy());
    }

    #[test]
    fn double_initialize_fails() {
        let mut m = KernelMachine::new(RealmId::Telum);
        m.initialize().unwrap();
        assert!(m.initialize().is_err());
    }

    // ── Test 2: Full happy-path pipeline ──────────────────────────────────

    #[test]
    fn happy_path_returns_to_idle() {
        let mut m = initialized_machine();
        m.simulate_happy_path_event();
        assert_eq!(m.phase, KernelPhase::Idle,
            "full happy path must end at Idle");
    }

    #[test]
    fn happy_path_visits_all_pipeline_phases() {
        let mut m = initialized_machine();
        m.simulate_happy_path_event();
        let phases = m.phases_visited();
        let required = [
            KernelPhase::ValidatingAbi, KernelPhase::ValidatingSchema,
            KernelPhase::ValidatingClock, KernelPhase::ValidatingCapability,
            KernelPhase::ValidatingCausal, KernelPhase::Deciding,
            KernelPhase::Applying, KernelPhase::Stamping, KernelPhase::Emitting,
        ];
        for phase in required {
            assert!(phases.contains(&phase), "happy path must visit {phase}");
        }
    }

    // ── Test 3: Rejection paths ────────────────────────────────────────────

    #[test]
    fn abi_failure_returns_to_idle() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::EventArrived);
        assert_eq!(m.phase, KernelPhase::ValidatingAbi);
        m.step(KernelInputKind::AbiFailed);
        assert_eq!(m.phase, KernelPhase::Idle);
    }

    #[test]
    fn capability_denied_returns_to_idle() {
        let mut m = initialized_machine();
        m.run([
            KernelInputKind::EventArrived,
            KernelInputKind::AbiValid, KernelInputKind::SchemaValid,
            KernelInputKind::ClockValid, KernelInputKind::CapabilityDenied,
        ]);
        assert_eq!(m.phase, KernelPhase::Idle);
    }

    // ── Test 4: Fault handling ────────────────────────────────────────────

    #[test]
    fn fault_from_processing_phase() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::EventArrived);
        m.step(KernelInputKind::FaultDetected);
        assert_eq!(m.phase, KernelPhase::Faulted);
        assert!(!m.is_healthy());
        assert_eq!(m.fault_count(), 1);
    }

    #[test]
    fn faulted_absorbs_all_normal_inputs() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::FaultDetected);
        assert_eq!(m.phase, KernelPhase::Faulted);
        // Further inputs don't change phase
        m.step(KernelInputKind::EventArrived);
        m.step(KernelInputKind::AbiValid);
        assert_eq!(m.phase, KernelPhase::Faulted);
    }

    #[test]
    fn recovery_path() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::FaultDetected);
        assert_eq!(m.phase, KernelPhase::Faulted);
        m.step(KernelInputKind::RecoveryAttempted);
        assert_eq!(m.phase, KernelPhase::Recovering);
        m.step(KernelInputKind::RecoverySucceeded);
        assert_eq!(m.phase, KernelPhase::Idle);
        assert!(m.is_healthy());
    }

    // ── Test 5: Replay arc ────────────────────────────────────────────────

    #[test]
    fn replay_confirms_identity() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::ReplayRequested);
        m.step(KernelInputKind::EventArrived);
        // Apply several events
        for _ in 0..5 { m.step(KernelInputKind::ReplayEventApplied); }
        m.step(KernelInputKind::ReplayComplete);
        assert_eq!(m.phase, KernelPhase::VerifyingIdentity);
        m.step(KernelInputKind::IdentityConfirmed);
        assert_eq!(m.phase, KernelPhase::Idle);
    }

    #[test]
    fn replay_diverged_faults_kernel() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::ReplayRequested);
        m.step(KernelInputKind::EventArrived);
        m.step(KernelInputKind::ReplayComplete);
        m.step(KernelInputKind::IdentityDiverged);
        assert_eq!(m.phase, KernelPhase::Faulted);
    }

    // ── Test 6: Seal ──────────────────────────────────────────────────────

    #[test]
    fn seal_is_terminal() {
        let mut m = initialized_machine();
        m.step(KernelInputKind::SealRequested);
        assert_eq!(m.phase, KernelPhase::Sealed);
        assert!(m.is_sealed());
        // Nothing changes Sealed
        m.step(KernelInputKind::EventArrived);
        assert_eq!(m.phase, KernelPhase::Sealed);
    }

    // ── Test 7: Invalid transition detection ──────────────────────────────

    #[test]
    fn invalid_transition_goes_to_faulted_and_records() {
        let mut m = initialized_machine();
        // AbiValid when Idle (no event in flight) is invalid
        let (phase, _effects) = m.step(KernelInputKind::AbiValid);
        assert_eq!(phase, KernelPhase::Faulted);
        assert!(!m.invalid_transitions().is_empty());
    }

    // ── Test 8: History is append-only ────────────────────────────────────

    #[test]
    fn history_is_append_only() {
        let mut m = initialized_machine();
        m.simulate_happy_path_event();
        let len_before = m.history_len();
        m.simulate_happy_path_event();
        let len_after = m.history_len();
        assert!(len_after > len_before, "history must grow");
        // Sequence numbers are monotone
        let seqs: Vec<u64> = m.history().iter().map(|r| r.seq).collect();
        for w in seqs.windows(2) { assert!(w[1] > w[0], "seq must be monotone"); }
    }

    // ── Test 9: Multiple events through one machine ───────────────────────

    #[test]
    fn three_events_processed_sequentially() {
        let mut m = initialized_machine();
        for _ in 0..3 {
            m.simulate_happy_path_event();
            assert_eq!(m.phase, KernelPhase::Idle, "must return to Idle after each event");
        }
        // processed_count increments on EmitComplete
        // (effects contain EventAccepted)
    }

    // ── Test 10: Transition table stats ───────────────────────────────────

    #[test]
    fn transition_table_is_consistent() {
        let stats = TransitionTableStats::compute();
        assert!(stats.phase_count > 0);
        assert!(stats.input_count > 0);
        assert!(stats.coverage_percent() > 0.0);
    }
}
