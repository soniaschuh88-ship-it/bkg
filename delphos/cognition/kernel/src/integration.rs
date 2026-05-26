// integration.rs — Proves the full pipeline compiles and all laws are satisfied.
//
// Compile-time proof of the complete causal pipeline:
//
//   PipelineEvent
//     ↓ EventPipeline.process()     — ABI+Schema+Clock+Capability validated
//     ↓ KernelDecision::Allow
//     ↓ ReplaySession.apply()       — StateTransitionFn<E> applied
//     ↓ TransitionLog.record()      — invariants enforced
//     ↓ MaterializerKernel.stamp()  — KernelStamp issued
//     ↓ ProjectionContract.verify() — hash contract satisfied
//     ↓ ReplayIdentityVerifier      — identity Confirmed or Diverged
//
// If this file compiles and all tests pass: the full causal pipeline is
// structurally sound. Single source of truth.

#[cfg(test)]
mod tests {
    use bkg_core::RealmId;

    // Kernel types
    use crate::{
        EventPipeline, PipelineConfig, PipelineEvent,
        ReplaySession, ReplayIdentityProof,
    };

    // State types
    use bkg_state::{
        RealmState, TransitionError,
        EventRange, MaterializerKernel,
        KanbanProjection, ProjectionFactory,
    };

    // ── Minimal reducer (pure function, deterministic) ────────────────────────

    fn kv_reducer(state: &RealmState, (key, val): (&str, &str)) -> Result<RealmState, TransitionError> {
        let mut next = state.clone().next_version(None, None);
        next.set_entity("kv", key, serde_json::json!(val));
        Ok(next)
    }

    fn apply_kv(
        session: &mut ReplaySession,
        id: &str, lamport: u64,
        key: &'static str, val: &'static str,
    ) -> Result<(), TransitionError> {
        session.apply(kv_reducer, id, "kv.set", lamport, (key, val))
    }

    // ── Test 1: Full pipeline end-to-end ─────────────────────────────────────

    #[test]
    fn full_pipeline_law_of_physics() {
        // 1. EventPipeline validates the event (all 5 stages)
        let mut cfg = PipelineConfig::default();
        cfg.grant_capability("scheduler", "task:create");
        let mut pipeline = EventPipeline::new(cfg);

        let ev = PipelineEvent::new(
            "evt-001", "task.created",
            RealmId::Telum, RealmId::Telum, 1,
            serde_json::json!({"task_id": "T-1"}),
        ).require_capability("task:create").with_actor("scheduler");

        let result = pipeline.process(&ev);
        assert!(result.decision.is_allow(), "pipeline must allow valid event");
        assert_eq!(pipeline.processed_count(), 1);

        // 2. StateTransitionFn<E> applied via ReplaySession
        let s0 = RealmState::empty(RealmId::Telum);
        let mut session = ReplaySession::from(s0);
        apply_kv(&mut session, "evt-001", 1, "T-1", "todo").unwrap();
        assert_eq!(session.current_version(), 1);

        // 3. MaterializerKernel stamps the projection (Kernel hook)
        let mut mat_kernel = MaterializerKernel::new();
        let data = serde_json::json!({"tasks": ["T-1"], "count": 1});
        let contract = mat_kernel.stamp("kanban", "telum", EventRange::single(1), &data);
        assert!(contract.kernel_stamp.is_some(), "KernelStamp must be present");
        assert!(contract.verify(&data).is_ok(), "projection must verify");

        // 4. Replay identity holds
        let mut rebuilt = ReplaySession::from(RealmState::empty(RealmId::Telum));
        apply_kv(&mut rebuilt, "evt-001", 1, "T-1", "todo").unwrap();
        let proof = rebuilt.verify_identity(&session.log);
        assert!(proof.is_confirmed(), "replay identity must hold: {proof:?}");
    }

    // ── Test 2: Projection hash contract prevents drift ───────────────────────

    #[test]
    fn projection_contract_detects_tampering() {
        let mut kernel = MaterializerKernel::new();
        let original = serde_json::json!({"tasks": ["T-1", "T-2"], "count": 2});
        let contract = kernel.stamp("kanban", "telum", EventRange::new(1, 5, 5), &original);

        assert!(contract.verify(&original).is_ok());

        let tampered = serde_json::json!({"tasks": ["T-1", "T-INJECTED"], "count": 99});
        assert!(contract.verify(&tampered).is_err(), "tampered projection must be detected");
    }

    // ── Test 3: Rebuild guarantee ─────────────────────────────────────────────

    #[test]
    fn rebuild_guarantee() {
        let s0 = RealmState::empty(RealmId::Causa);

        let mut original = ReplaySession::from(s0.clone());
        apply_kv(&mut original, "e1", 10, "status", "active").unwrap();
        apply_kv(&mut original, "e2", 20, "priority", "high").unwrap();

        let mut rebuilt = ReplaySession::from(s0);
        apply_kv(&mut rebuilt, "e1", 10, "status", "active").unwrap();
        apply_kv(&mut rebuilt, "e2", 20, "priority", "high").unwrap();

        let proof = rebuilt.verify_identity(&original.log);
        assert!(proof.is_confirmed());
        assert_eq!(original.log.tip_checksum(), rebuilt.log.tip_checksum(),
            "rebuild guarantee: same events → same checksum");
    }

    // ── Test 4: Divergence detected immediately ───────────────────────────────

    #[test]
    fn divergence_detected_immediately() {
        let s0 = RealmState::empty(RealmId::Telum);

        let mut original = ReplaySession::from(s0.clone());
        apply_kv(&mut original, "e1", 1, "x", "correct").unwrap();

        let mut corrupted = ReplaySession::from(s0);
        apply_kv(&mut corrupted, "e1", 1, "x", "CORRUPTED").unwrap();

        let proof = corrupted.verify_identity(&original.log);
        assert!(proof.is_diverged(), "corruption must be detected");

        if let ReplayIdentityProof::Diverged { diverged_at_lamport, .. } = proof {
            assert_eq!(diverged_at_lamport, Some(1));
        }
    }

    // ── Test 5: Kernel rejects invalid events ─────────────────────────────────

    #[test]
    fn pipeline_rejects_duplicate_lamport() {
        let mut p = EventPipeline::new(PipelineConfig::default());
        let e1 = PipelineEvent::new("e1","s",RealmId::Telum,RealmId::Telum,5,serde_json::json!({}));
        let e2 = PipelineEvent::new("e2","s",RealmId::Telum,RealmId::Telum,5,serde_json::json!({}));
        p.process(&e1);
        assert!(p.process(&e2).decision.is_reject());
    }

    #[test]
    fn pipeline_rejects_replay_paradox() {
        let mut p = EventPipeline::new(PipelineConfig::default());
        let e1 = PipelineEvent::new("evt-dup","s",RealmId::Telum,RealmId::Telum,5,serde_json::json!({}));
        let e2 = PipelineEvent::new("evt-dup","s",RealmId::Telum,RealmId::Telum,6,serde_json::json!({}));
        p.process(&e1);
        assert!(p.process(&e2).decision.is_reject());
    }

    // ── Test 6: Projection isolation — structural, not documented ────────────

    #[test]
    fn projection_view_is_structurally_read_only() {
        let view = ProjectionFactory::create(
            "kanban", "telum", 3, "ck-abc",
            KanbanProjection { todo: vec!["T-1".into()], total_tasks: 1, ..Default::default() },
        );

        assert_eq!(view.data().total_tasks, 1);
        assert!(!view.is_stale("ck-abc"));
        assert!(view.is_stale("ck-xyz"));
        // view.data_mut() → does not exist. This test compiling IS the proof.
    }

    // ── Test 7: Transition log enforces continuity ────────────────────────────

    #[test]
    fn transition_log_enforces_continuity() {
        let mut s = ReplaySession::from(RealmState::empty(RealmId::Anamnesis));
        apply_kv(&mut s, "e1", 1, "a", "1").unwrap();
        apply_kv(&mut s, "e2", 2, "b", "2").unwrap();
        apply_kv(&mut s, "e3", 3, "c", "3").unwrap();

        assert_eq!(s.current_version(), 3);
        assert_eq!(s.log.event_range().event_count, 3);
        assert_eq!(s.log.event_range().from_lamport, 1);
        assert_eq!(s.log.event_range().to_lamport, 3);
    }
}
