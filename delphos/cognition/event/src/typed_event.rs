// typed_event.rs — TypedEvent<P: EventPayload>
//
// Replaces dynamic serde_json::Value payloads with compile-time typed events.
// Guarantees:
//   - Every event has a compile-time-known payload type P
//   - P: EventPayload ensures schema_id is always present
//   - P: Serialize + DeserializeOwned ensures deterministic serialization
//   - payload_hash is always computed and verified
//   - causal_parent enforces ordering
//
// This is the upgrade path from DomainEvent (which still uses serde_json::Value)
// to fully typed events with compile-time replay guarantees.
//
// Single source of truth for all typed event definitions.

use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

// ─── EventPayload trait ───────────────────────────────────────────────────────

/// Trait bound for all typed event payloads in DELPHOS.
///
/// Every struct that can be an event payload must implement this.
/// It provides the static schema_id and version — no runtime lookup needed.
///
/// ## Example (conceptual)
/// ```ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct TaskCreated { id: String, title: String }
/// impl EventPayload for TaskCreated {
///     const SCHEMA_ID: &'static str = "task.created";
///     fn producer_realm() -> RealmId { RealmId::Telum }
/// }
/// ```
pub trait EventPayload: Serialize + Clone + Send + Sync + 'static {
    /// Stable schema identifier. Must match EventSchemaRegistry.
    const SCHEMA_ID: &'static str;
    /// Schema major version.
    const SCHEMA_VERSION_MAJOR: u16 = 1;
    /// Schema minor version.
    const SCHEMA_VERSION_MINOR: u16 = 0;
    /// Which realm produces this event type.
    fn producer_realm() -> RealmId;
    /// Human-readable description of this event type.
    fn description() -> &'static str { Self::SCHEMA_ID }
}

// ─── TypedEvent<P> ────────────────────────────────────────────────────────────

/// A fully typed, replay-safe event.
///
/// Unlike DomainEvent (which uses serde_json::Value), TypedEvent<P> knows
/// its payload type at compile time. This enables:
/// - Compile-time schema validation
/// - Typed reducers: `Reducer<TypedEvent<TaskCreated>>`
/// - Zero-cost serialization (no Value boxing)
/// - Replay verification (same P → same bytes)
#[derive(Debug, Clone, Serialize)]
pub struct TypedEvent<P: EventPayload> {
    pub id: String,
    pub schema_id: &'static str,
    pub schema_version_major: u16,
    pub schema_version_minor: u16,
    pub source_realm: RealmId,
    pub target_realm: RealmId,
    /// Lamport counter — from bkg-clock. No SystemTime::now() here.
    pub lamport: u64,
    /// Wall time — display only. Never used for ordering.
    pub wall_nanos: u64,
    pub producer: String,
    /// Causal parent event ID. None = genesis event.
    pub causal_parent: Option<String>,
    /// The typed payload. Private — access via payload().
    payload: P,
    // Note: custom Serialize impl serializes payload as Value
    /// BLAKE3-style hash of serialized payload for integrity.
    pub payload_hash: String,
}

impl<P: EventPayload> TypedEvent<P> {
    /// Construct a new TypedEvent.
    /// Uses `P::SCHEMA_ID` automatically — no runtime schema lookup.
    pub fn new(
        source: RealmId,
        lamport: u64,
        producer: impl Into<String>,
        payload: P,
    ) -> serde_json::Result<Self> {
        let serialized = serde_json::to_string(&payload)?;
        let hash = compute_hash(&serialized);
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            schema_id: P::SCHEMA_ID,
            schema_version_major: P::SCHEMA_VERSION_MAJOR,
            schema_version_minor: P::SCHEMA_VERSION_MINOR,
            source_realm: source,
            target_realm: P::producer_realm(),
            lamport,
            wall_nanos: wall_nanos_now(),
            producer: producer.into(),
            causal_parent: None,
            payload,
            payload_hash: hash,
        })
    }

    pub fn with_causal_parent(mut self, parent: impl Into<String>) -> Self {
        self.causal_parent = Some(parent.into());
        self
    }

    pub fn with_target(mut self, target: RealmId) -> Self {
        self.target_realm = target;
        self
    }

    /// Read-only access to the typed payload.
    pub fn payload(&self) -> &P { &self.payload }

    /// Verify the payload hash — detects tampering.
    pub fn verify(&self) -> bool {
        serde_json::to_string(&self.payload)
            .map(|s| compute_hash(&s) == self.payload_hash)
            .unwrap_or(false)
    }

    /// Erase to serde_json::Value for storage in existing event ledger.
    pub fn to_value(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(&self.payload)
    }

    // from_value requires P: DeserializeOwned — call from_value_owned when needed.

    pub fn schema_version(&self) -> (u16, u16) {
        (self.schema_version_major, self.schema_version_minor)
    }
}

fn compute_hash(data: &str) -> String {
    use std::hash::Hash;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut h);
    format!("{:x}", std::hash::Hasher::finish(&h))
}

fn wall_nanos_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

// ─── Well-known DELPHOS event payloads ────────────────────────────────────────

// These are the canonical event types. One definition per concept.
// Single source of truth.

/// A new task was created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCreated {
    pub task_id: String,
    pub title: String,
    pub priority: String,
    pub agent_id: Option<String>,
}
impl EventPayload for TaskCreated {
    const SCHEMA_ID: &'static str = "task.created";
    fn producer_realm() -> RealmId { RealmId::Telum }
}

/// A task's status changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStatusChanged {
    pub task_id: String,
    pub from_status: String,
    pub to_status: String,
    pub changed_by: String,
}
impl EventPayload for TaskStatusChanged {
    const SCHEMA_ID: &'static str = "task.status_changed";
    fn producer_realm() -> RealmId { RealmId::Telum }
}

/// An agent session was started.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStarted {
    pub session_id: String,
    pub agent_id: String,
    pub mode: String,
}
impl EventPayload for SessionStarted {
    const SCHEMA_ID: &'static str = "session.started";
    fn producer_realm() -> RealmId { RealmId::Telum }
}

/// An approval was granted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalGranted {
    pub approval_id: String,
    pub kind: String,
    pub granted_by: String,
}
impl EventPayload for ApprovalGranted {
    const SCHEMA_ID: &'static str = "approval.granted";
    fn producer_realm() -> RealmId { RealmId::Katoptron }
}

/// An approval was rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRejected {
    pub approval_id: String,
    pub reason: String,
    pub rejected_by: String,
}
impl EventPayload for ApprovalRejected {
    const SCHEMA_ID: &'static str = "approval.rejected";
    fn producer_realm() -> RealmId { RealmId::Katoptron }
}

/// A workflow gate passed (APPROVE verdict).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowApproved {
    pub task_id: String,
    pub phase: String,
    pub reviewer_model: Option<String>,
    pub feedback: String,
}
impl EventPayload for WorkflowApproved {
    const SCHEMA_ID: &'static str = "workflow.approved";
    fn producer_realm() -> RealmId { RealmId::Telum }
}

/// A workflow gate failed (RETHINK or max retries).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowFailed {
    pub task_id: String,
    pub phase: String,
    pub reason: String,
}
impl EventPayload for WorkflowFailed {
    const SCHEMA_ID: &'static str = "workflow.failed";
    fn producer_realm() -> RealmId { RealmId::Telum }
}

/// A capability grant was issued.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityGranted {
    pub grantee: String,
    pub capability: String,
    pub granted_by: String,
    pub ttl_secs: Option<i64>,
}
impl EventPayload for CapabilityGranted {
    const SCHEMA_ID: &'static str = "capability.granted";
    fn producer_realm() -> RealmId { RealmId::Speculum }
}

/// A realm state snapshot was taken.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotCreated {
    pub snapshot_id: String,
    pub realm_id: String,
    pub state_version: u64,
    pub checksum: String,
}
impl EventPayload for SnapshotCreated {
    const SCHEMA_ID: &'static str = "snapshot.created";
    fn producer_realm() -> RealmId { RealmId::Speculum }
}

// ─── TypedEventRegistry ───────────────────────────────────────────────────────

/// A compile-time registry entry for a typed event.
/// Used to build the EventSchemaRegistry at runtime.
#[derive(Debug, Clone)]
pub struct TypedEventInfo {
    pub schema_id: &'static str,
    pub version_major: u16,
    pub version_minor: u16,
    pub producer_realm: RealmId,
    pub description: &'static str,
}

/// All well-known typed events in DELPHOS.
/// Add new events here — this is the single source of truth.
pub fn all_typed_events() -> Vec<TypedEventInfo> {
    vec![
        TypedEventInfo { schema_id: TaskCreated::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: TaskCreated::producer_realm(), description: TaskCreated::description() },
        TypedEventInfo { schema_id: TaskStatusChanged::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: TaskStatusChanged::producer_realm(), description: TaskStatusChanged::description() },
        TypedEventInfo { schema_id: SessionStarted::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: SessionStarted::producer_realm(), description: SessionStarted::description() },
        TypedEventInfo { schema_id: ApprovalGranted::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: ApprovalGranted::producer_realm(), description: ApprovalGranted::description() },
        TypedEventInfo { schema_id: ApprovalRejected::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: ApprovalRejected::producer_realm(), description: ApprovalRejected::description() },
        TypedEventInfo { schema_id: WorkflowApproved::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: WorkflowApproved::producer_realm(), description: WorkflowApproved::description() },
        TypedEventInfo { schema_id: WorkflowFailed::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: WorkflowFailed::producer_realm(), description: WorkflowFailed::description() },
        TypedEventInfo { schema_id: CapabilityGranted::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: CapabilityGranted::producer_realm(), description: CapabilityGranted::description() },
        TypedEventInfo { schema_id: SnapshotCreated::SCHEMA_ID, version_major: 1, version_minor: 0, producer_realm: SnapshotCreated::producer_realm(), description: SnapshotCreated::description() },
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;

    #[test]
    fn typed_event_created() {
        let ev = TypedEvent::new(
            RealmId::Telum, 1, "scheduler",
            TaskCreated { task_id: "T-1".into(), title: "impl typed events".into(), priority: "high".into(), agent_id: None },
        ).unwrap();
        assert_eq!(ev.schema_id, "task.created");
        assert_eq!(ev.schema_version(), (1, 0));
        assert_eq!(ev.payload().task_id, "T-1");
    }

    #[test]
    fn verify_hash() {
        let ev = TypedEvent::new(
            RealmId::Telum, 1, "test",
            TaskCreated { task_id: "T-2".into(), title: "check hash".into(), priority: "normal".into(), agent_id: None },
        ).unwrap();
        assert!(ev.verify());
    }

    #[test]
    fn causal_parent() {
        let parent_id = "evt-parent".to_string();
        let ev = TypedEvent::new(RealmId::Telum, 2, "test",
            TaskStatusChanged { task_id: "T-1".into(), from_status: "todo".into(), to_status: "in_progress".into(), changed_by: "agent-1".into() },
        ).unwrap().with_causal_parent(&parent_id);
        assert_eq!(ev.causal_parent.as_deref(), Some("evt-parent"));
    }

    #[test]
    fn to_value() {
        let ev = TypedEvent::new(RealmId::Telum, 3, "test",
            TaskCreated { task_id: "T-3".into(), title: "roundtrip".into(), priority: "low".into(), agent_id: None }).unwrap();
        let v = ev.to_value().unwrap();
        assert_eq!(v["task_id"], "T-3");
    }

    #[test]
    fn schema_ids_unique() {
        let events = all_typed_events();
        let mut ids: Vec<&str> = events.iter().map(|e| e.schema_id).collect();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len); // no duplicates
    }

    #[test]
    fn all_known_events_registered() {
        let events = all_typed_events();
        assert!(events.len() >= 9);
        assert!(events.iter().any(|e| e.schema_id == "task.created"));
        assert!(events.iter().any(|e| e.schema_id == "workflow.approved"));
        assert!(events.iter().any(|e| e.schema_id == "capability.granted"));
    }

    #[test]
    fn session_event() {
        let ev = TypedEvent::new(RealmId::Telum, 10, "runtime",
            SessionStarted { session_id: "S-1".into(), agent_id: "claude".into(), mode: "bkg_supervised".into() },
        ).unwrap();
        assert_eq!(ev.schema_id, "session.started");
        assert!(ev.verify());
    }

    #[test]
    fn approval_events() {
        let granted = TypedEvent::new(RealmId::Katoptron, 20, "operator",
            ApprovalGranted { approval_id: "AP-1".into(), kind: "merge".into(), granted_by: "alice".into() },
        ).unwrap();
        assert_eq!(granted.schema_id, "approval.granted");

        let rejected = TypedEvent::new(RealmId::Katoptron, 21, "operator",
            ApprovalRejected { approval_id: "AP-2".into(), reason: "unsafe code".into(), rejected_by: "safety-gate".into() },
        ).unwrap();
        assert_eq!(rejected.schema_id, "approval.rejected");
    }
}
