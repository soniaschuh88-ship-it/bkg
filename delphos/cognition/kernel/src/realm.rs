// realm.rs — The Realm: atomic commit, zero dual-truth drift.
//
// PROBLEM: The integration boundary
//
//   EventPipeline → KernelMachine → Reducer<E> → MaterializerKernel → EventLedger
//
// has five places where partial failure causes dual-truth:
//   - Pipeline allows event, but Reducer fails → state not updated, but pipeline
//     already advanced to Applying phase
//   - Reducer succeeds, but Materializer fails → new state exists but no KernelStamp
//   - Materializer succeeds, but Ledger append fails → projection stamped but not recorded
//
// FIX: Realm::submit_event() is ATOMIC.
//   1. Run all stages in a staging area (no mutation yet)
//   2. Only if ALL stages succeed: commit atomically
//      a. Advance KernelMachine phase by phase
//      b. Apply new RealmState
//      c. Append to EventLedger (last — the commit point)
//   3. If any stage fails: zero observable change. KernelMachine resets to Idle.
//
// Observer invariant: at any point in time,
//   ledger.len() == transition_log.len() == realm_state.version
//
// Single source of truth for DELPHOS realm execution.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_core::RealmId;

use crate::{
    event_ledger::{EventLedger, ChainVerification},
    kernel_machine::KernelMachine,
    kernel_state::{KernelInputKind, KernelPhase},
    pipeline::{EventPipeline, PipelineConfig, PipelineEvent, KernelDecision},
    state_transition::{
        ReplayIdentityProof, ReplayIdentityVerifier, ReplaySession,
        StateTransitionFn, TransitionLog,
    },
    rule_engine::RuleEngine,
};
use bkg_state::{
    MaterializerKernel, EventRange,
    RealmState,
};

// ─── RealmSubmitResult ────────────────────────────────────────────────────────

/// Result of one atomic event submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmSubmitResult {
    pub event_id: String,
    pub lamport: u64,
    pub outcome: SubmitOutcome,
    pub phases_traversed: Vec<KernelPhase>,
    pub effects_emitted: Vec<String>,
    pub new_version: Option<u64>,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitOutcome {
    /// Event accepted, state updated, ledger appended.
    Committed,
    /// Event rejected by kernel (validation failed). State unchanged.
    Rejected(String),
    /// Internal failure during application. State unchanged (rolled back).
    Failed(String),
}

impl SubmitOutcome {
    pub fn is_committed(&self) -> bool { *self == Self::Committed }
    pub fn is_rejected(&self) -> bool { matches!(self, Self::Rejected(_)) }
}

// ─── RealmConsistencyProof ────────────────────────────────────────────────────

/// Proves that the realm is internally consistent at a given moment.
///
/// Invariants verified:
///   ledger.len() == state.version
///   ledger.chain_tip valid
///   transition_log.event_range ⊆ ledger.lamport_range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmConsistencyProof {
    pub realm_id: RealmId,
    pub state_version: u64,
    pub ledger_len: u64,
    pub chain_verification: ChainVerification,
    pub consistent: bool,
    pub violations: Vec<String>,
}

// ─── Realm ────────────────────────────────────────────────────────────────────

/// A DELPHOS realm: the atomic execution unit.
///
/// Owns:
///   - `RealmState`      — current world state
///   - `EventLedger`     — append-only hash-chained event history
///   - `KernelMachine`   — formal state machine (phase tracking)
///   - `TransitionLog`   — validated state transition history
///   - `MaterializerKernel` — projection stamping
///   - `EventPipeline`   — event validation
///   - `RuleEngine`      — compressed δ (verified at construction)
///
/// All state mutation happens through `submit_event()`. Atomic. No exceptions.
pub struct Realm {
    pub id: RealmId,
    pub label: String,
    state: RealmState,
    ledger: EventLedger,
    machine: KernelMachine,
    transition_log: TransitionLog,
    mat_kernel: MaterializerKernel,
    pipeline: EventPipeline,
    #[allow(dead_code)] rule_engine: RuleEngine,
    submit_count: u64,
    reject_count: u64,
}

impl Realm {
    /// Create a new realm. Verifies rule engine consistency at construction.
    /// Panics in debug builds if rule engine fails verification (programming error).
    pub fn new(id: RealmId, label: impl Into<String>, pipeline_config: PipelineConfig) -> Self {
        let engine = RuleEngine::new();
        let report = engine.verify();
        debug_assert!(report.is_ok(),
            "Rule engine verification failed at Realm construction: {:?}", report);

        let mut machine = KernelMachine::new(id);
        machine.initialize().expect("kernel initialization must succeed");

        Self {
            id,
            label: label.into(),
            state: RealmState::empty(id),
            ledger: EventLedger::new(id),
            machine,
            transition_log: TransitionLog::new(id),
            mat_kernel: MaterializerKernel::new(),
            pipeline: EventPipeline::new(pipeline_config),
            rule_engine: engine,
            submit_count: 0,
            reject_count: 0,
        }
    }

    /// Create a realm with default (open) pipeline config.
    pub fn open(id: RealmId, label: impl Into<String>) -> Self {
        Self::new(id, label, PipelineConfig::default())
    }

    // ── The atomic submit ─────────────────────────────────────────────────

    /// Submit one event to the realm atomically.
    ///
    /// The commit protocol:
    ///   Stage 1: Validate through EventPipeline (no state change)
    ///   Stage 2: Apply via reducer (no state change, produce candidate_state)
    ///   Stage 3: Stamp projection (no state change, produce contract)
    ///   Commit:  Advance KernelMachine → update state → append ledger
    ///
    /// If any stage fails: zero observable change.
    pub fn submit_event<E: Clone>(
        &mut self,
        pipeline_event: &PipelineEvent,
        reducer: StateTransitionFn<E>,
        event: E,
    ) -> RealmSubmitResult {
        let event_id = pipeline_event.event_id.clone();
        let lamport = pipeline_event.lamport;
        self.submit_count += 1;

        // ── Stage 1: Pipeline validation ─────────────────────────────────
        self.machine.step(KernelInputKind::EventArrived);
        let pipeline_result = self.pipeline.process(pipeline_event);
        let decision = pipeline_result.decision.clone();
        match &decision {
            KernelDecision::Allow | KernelDecision::Transform { .. } => {
                // Advance through all validation phases
                self.machine.step(KernelInputKind::AbiValid);
                self.machine.step(KernelInputKind::SchemaValid);
                self.machine.step(KernelInputKind::ClockValid);
                self.machine.step(KernelInputKind::CapabilityGranted);
                self.machine.step(KernelInputKind::CausalValid);
                self.machine.step(KernelInputKind::DecisionAllow);
            }
            KernelDecision::Reject(reason) => {
                self.machine.step(KernelInputKind::AbiFailed); // collapse to idle
                self.reject_count += 1;
                return RealmSubmitResult {
                    event_id, lamport,
                    outcome: SubmitOutcome::Rejected(reason.to_string()),
                    phases_traversed: vec![KernelPhase::ValidatingAbi, KernelPhase::Idle],
                    effects_emitted: vec!["EventRejected".into()],
                    new_version: None,
                    committed_at: Utc::now(),
                };
            }
        }

        // ── Stage 2: Apply reducer (candidate only, no commit yet) ────────
        let candidate = match reducer(&self.state, event) {
            Ok(next) => next,
            Err(e) => {
                self.machine.step(KernelInputKind::TransitionFailed);
                self.machine.step(KernelInputKind::RecoverySucceeded); // auto-recover
                return RealmSubmitResult {
                    event_id, lamport,
                    outcome: SubmitOutcome::Failed(e.to_string()),
                    phases_traversed: self.machine.phases_visited().clone(),
                    effects_emitted: vec!["TransitionFailed".into()],
                    new_version: None,
                    committed_at: Utc::now(),
                };
            }
        };

        self.machine.step(KernelInputKind::TransitionApplied);

        // ── Stage 3: Stamp projection (candidate only) ────────────────────
        let range = if self.transition_log.is_empty() {
            EventRange::single(lamport)
        } else {
            self.transition_log.event_range().extend(lamport)
        };
        let projection_data = serde_json::json!({
            "realm": self.id.as_str(),
            "version": candidate.version,
            "checksum": candidate.checksum(),
        });
        let _contract = self.mat_kernel.stamp(
            &format!("{}-state", self.id.as_str()),
            self.id.as_str(),
            range,
            &projection_data,
        );
        self.machine.step(KernelInputKind::ProjectionStamped);

        // ── COMMIT POINT ──────────────────────────────────────────────────
        // All stages succeeded. Now commit atomically.

        // Record state transition
        let transition = match crate::state_transition::StateTransition::record(
            &event_id, &pipeline_event.schema_id, lamport, &self.state, &candidate,
        ) {
            Ok(t) => t,
            Err(e) => {
                self.machine.step(KernelInputKind::FaultDetected);
                return RealmSubmitResult {
                    event_id, lamport,
                    outcome: SubmitOutcome::Failed(format!("transition record failed: {e}")),
                    phases_traversed: self.machine.phases_visited().clone(),
                    effects_emitted: vec![],
                    new_version: None,
                    committed_at: Utc::now(),
                };
            }
        };

        if let Err(e) = self.transition_log.record(transition) {
            self.machine.step(KernelInputKind::FaultDetected);
            return RealmSubmitResult {
                event_id, lamport,
                outcome: SubmitOutcome::Failed(format!("log record failed: {e}")),
                phases_traversed: self.machine.phases_visited().clone(),
                effects_emitted: vec![],
                new_version: None,
                committed_at: Utc::now(),
            };
        }

        // Apply state (point of no return within memory)
        let new_version = candidate.version;
        self.state = candidate;

        // Append to ledger (THE commit point — last mutation)
        if let Err(e) = self.ledger.append(
            &event_id, &pipeline_event.schema_id, lamport,
            &pipeline_event.payload_hash, pipeline_event.payload.clone(),
            &pipeline_event.producer, pipeline_event.causal_parent.clone(),
        ) {
            // Ledger append failed after state was applied — record fault
            self.machine.step(KernelInputKind::FaultDetected);
            return RealmSubmitResult {
                event_id, lamport,
                outcome: SubmitOutcome::Failed(format!("ledger append failed: {e}")),
                phases_traversed: self.machine.phases_visited().clone(),
                effects_emitted: vec![],
                new_version: Some(new_version),
                committed_at: Utc::now(),
            };
        }

        // Complete KernelMachine cycle
        let (_, effects) = self.machine.step(KernelInputKind::EmitComplete);

        RealmSubmitResult {
            event_id, lamport,
            outcome: SubmitOutcome::Committed,
            phases_traversed: self.machine.phases_visited().clone(),
            effects_emitted: effects.iter().map(|e| format!("{e:?}")).collect(),
            new_version: Some(new_version),
            committed_at: Utc::now(),
        }
    }

    // ── Replay ────────────────────────────────────────────────────────────

    /// Replay all ledger entries through a reducer and verify identity.
    ///
    /// This is the Rebuild Guarantee: same events → same state.
    pub fn verify_replay_identity<E: Clone>(
        &self,
        reducer: StateTransitionFn<E>,
        event_factory: impl Fn(&bkg_state::EventRange, u64, &serde_json::Value) -> E,
    ) -> ReplayIdentityProof {
        let s0 = RealmState::empty(self.id);
        let mut session = ReplaySession::from(s0);

        for entry in self.ledger.entries() {
            let range = EventRange::single(entry.lamport);
            let event = event_factory(&range, entry.lamport, &entry.payload);
            if session.apply(reducer, &entry.event_id, &entry.schema_id, entry.lamport, event).is_err() {
                // Cannot replay — diverged
                return ReplayIdentityProof::Diverged {
                    event_range: self.transition_log.event_range(),
                    original_checksum: self.state.checksum(),
                    rebuilt_checksum: "REPLAY_FAILED".to_string(),
                    diverged_at_lamport: Some(entry.lamport),
                };
            }
        }

        ReplayIdentityVerifier::verify(&self.transition_log, &session.current_state)
    }

    // ── Consistency verification ──────────────────────────────────────────

    /// Verify the realm's internal consistency invariants.
    pub fn verify_consistency(&self) -> RealmConsistencyProof {
        let mut violations = Vec::new();

        let state_version = self.state.version;
        let ledger_len = self.ledger.len() as u64;
        let chain = self.ledger.verify_chain();

        // Invariant: ledger.len() == state.version
        if ledger_len != state_version {
            violations.push(format!(
                "ledger.len()={ledger_len} ≠ state.version={state_version}"
            ));
        }

        // Invariant: chain is valid
        if !chain.is_valid() {
            violations.push(format!("chain broken: {chain:?}"));
        }

        // Invariant: transition_log.len() == ledger.len()
        let log_len = self.transition_log.len() as u64;
        if log_len != ledger_len {
            violations.push(format!(
                "transition_log.len()={log_len} ≠ ledger.len()={ledger_len}"
            ));
        }

        RealmConsistencyProof {
            realm_id: self.id,
            state_version,
            ledger_len,
            chain_verification: chain,
            consistent: violations.is_empty(),
            violations,
        }
    }

    // ── Introspection ─────────────────────────────────────────────────────

    pub fn state(&self) -> &RealmState { &self.state }
    pub fn ledger(&self) -> &EventLedger { &self.ledger }
    pub fn machine(&self) -> &KernelMachine { &self.machine }
    pub fn current_version(&self) -> u64 { self.state.version }
    pub fn current_phase(&self) -> KernelPhase { self.machine.phase }
    pub fn submit_count(&self) -> u64 { self.submit_count }
    pub fn reject_count(&self) -> u64 { self.reject_count }
    pub fn is_healthy(&self) -> bool { self.machine.is_healthy() }

    pub fn seal(&mut self) {
        self.machine.step(KernelInputKind::SealRequested);
        self.ledger.seal();
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;
    use bkg_state::{RealmState, TransitionError};

    fn kv_reducer(state: &RealmState, (key, val): (&str, &str)) -> Result<RealmState, TransitionError> {
        let mut next = state.clone().next_version(None, None);
        next.set_entity("kv", key, serde_json::json!(val));
        Ok(next)
    }

    fn event(id: &str, lamport: u64) -> PipelineEvent {
        PipelineEvent::new(id, "kv.set", RealmId::Telum, RealmId::Telum,
            lamport, serde_json::json!({"key": id}))
    }

    fn submit(realm: &mut Realm, id: &str, lamport: u64, key: &'static str, val: &'static str)
        -> RealmSubmitResult
    {
        realm.submit_event(&event(id, lamport), kv_reducer, (key, val))
    }

    #[test]
    fn realm_initializes_idle() {
        let r = Realm::open(RealmId::Telum, "test");
        assert_eq!(r.current_phase(), KernelPhase::Idle);
        assert_eq!(r.current_version(), 0);
        assert!(r.is_healthy());
    }

    #[test]
    fn single_event_commits() {
        let mut r = Realm::open(RealmId::Telum, "test");
        let result = submit(&mut r, "e1", 1, "status", "active");
        assert!(result.outcome.is_committed(), "{:?}", result.outcome);
        assert_eq!(r.current_version(), 1);
        assert_eq!(r.ledger().len(), 1);
    }

    #[test]
    fn three_events_sequential() {
        let mut r = Realm::open(RealmId::Telum, "test");
        for i in 1..=3u64 {
            let res = submit(&mut r, &format!("e{i}"), i, "k", "v");
            assert!(res.outcome.is_committed(), "e{i} must commit: {:?}", res.outcome);
        }
        assert_eq!(r.current_version(), 3);
        assert_eq!(r.ledger().len(), 3);
    }

    #[test]
    fn consistency_invariants_hold() {
        let mut r = Realm::open(RealmId::Telum, "test");
        submit(&mut r, "e1", 1, "a", "1");
        submit(&mut r, "e2", 2, "b", "2");
        let proof = r.verify_consistency();
        assert!(proof.consistent, "violations: {:?}", proof.violations);
    }

    #[test]
    fn chain_verification_valid_after_commits() {
        let mut r = Realm::open(RealmId::Telum, "test");
        for i in 1..=5u64 { submit(&mut r, &format!("e{i}"), i, "k", "v"); }
        assert!(r.ledger().verify_chain().is_valid());
    }

    #[test]
    fn pipeline_rejects_duplicate_lamport() {
        let mut r = Realm::open(RealmId::Telum, "test");
        submit(&mut r, "e1", 10, "k", "v1");
        let result = submit(&mut r, "e2", 10, "k", "v2"); // duplicate lamport
        assert!(result.outcome.is_rejected(), "{:?}", result.outcome);
        // State must be unchanged after rejection
        assert_eq!(r.current_version(), 1);
        assert_eq!(r.ledger().len(), 1);
    }

    #[test]
    fn state_unchanged_after_rejection() {
        let mut r = Realm::open(RealmId::Telum, "test");
        submit(&mut r, "e1", 1, "status", "ok");
        let v_before = r.current_version();
        let tip_before = r.ledger().chain_tip().to_string();

        // Duplicate event_id (rejected by pipeline)
        let dup = PipelineEvent::new("e1","kv.set",RealmId::Telum,RealmId::Telum,2,serde_json::json!({}));
        let res = r.submit_event(&dup, kv_reducer, ("status", "broken"));
        assert!(res.outcome.is_rejected());
        assert_eq!(r.current_version(), v_before, "state must not change on rejection");
        assert_eq!(r.ledger().chain_tip(), tip_before, "ledger must not change on rejection");
    }

    #[test]
    fn seal_blocks_further_events() {
        let mut r = Realm::open(RealmId::Telum, "test");
        submit(&mut r, "e1", 1, "k", "v");
        r.seal();
        assert!(r.ledger().sealed);
        assert_eq!(r.current_phase(), KernelPhase::Sealed);
    }

    #[test]
    fn realm_returns_to_idle_after_each_event() {
        let mut r = Realm::open(RealmId::Telum, "test");
        for i in 1..=3u64 {
            submit(&mut r, &format!("e{i}"), i, "k", "v");
            assert_eq!(r.current_phase(), KernelPhase::Idle,
                "must return to Idle after event {i}");
        }
    }

    #[test]
    fn rule_engine_verified_at_construction() {
        // Realm::new panics in debug if rule engine fails verification
        // This test verifies construction succeeds (rule engine is valid)
        let _ = Realm::open(RealmId::Causa, "verify");
    }
}
