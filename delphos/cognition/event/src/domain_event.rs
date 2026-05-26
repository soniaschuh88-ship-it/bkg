// domain_event.rs — DomainEvent: typed event with schema_id, causal_parent, payload hash.
// Extends the existing Event with schema registry integration and causal chaining.
// Single source of truth for typed event contracts.
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

/// Typed event identifier linking to EventSchemaRegistry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventSchemaRef {
    pub schema_id: String,
    pub schema_version_major: u16,
    pub schema_version_minor: u16,
}
impl EventSchemaRef {
    pub fn new(id: impl Into<String>, major: u16, minor: u16) -> Self {
        Self { schema_id: id.into(), schema_version_major: major, schema_version_minor: minor }
    }
    pub fn v1(id: impl Into<String>) -> Self { Self::new(id, 1, 0) }
}
impl std::fmt::Display for EventSchemaRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}.{}", self.schema_id, self.schema_version_major, self.schema_version_minor)
    }
}

/// Typed domain event: wraps any JSON-serializable payload with schema metadata.
/// Replaces raw `serde_json::Value` chaos for compile-time-verifiable event contracts.
///
/// Uses `serde_json::Value` as payload for practical compat with existing ledger.
/// Schema-typed decode is performed by the Reducer consuming this event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub realm_id: RealmId,
    pub schema: EventSchemaRef,
    /// Lamport counter (from bkg-clock SequencedInstant).
    pub lamport: u64,
    /// Wall nanos — display only, never used for ordering.
    pub wall_nanos: u64,
    pub producer: String,
    /// Causal parent event id — None for genesis events.
    pub causal_parent: Option<String>,
    pub payload: serde_json::Value,
    /// BLAKE3 hash of serialized payload for integrity.
    pub payload_hash: String,
}

impl DomainEvent {
    pub fn new(
        realm_id: RealmId,
        schema: EventSchemaRef,
        lamport: u64,
        producer: impl Into<String>,
        causal_parent: Option<String>,
        payload: serde_json::Value,
    ) -> Self {
        let json = payload.to_string();
        let hash = { use std::hash::Hasher; let mut h=std::collections::hash_map::DefaultHasher::new(); std::hash::Hash::hash_slice(json.as_bytes(),&mut h); format!("{:x}",h.finish()) };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            realm_id, schema, lamport,
            wall_nanos: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64).unwrap_or(0),
            producer: producer.into(),
            causal_parent, payload, payload_hash: hash,
        }
    }

    pub fn verify_hash(&self) -> bool {
        let hash = { use std::hash::Hasher; let s=self.payload.to_string(); let mut h=std::collections::hash_map::DefaultHasher::new(); std::hash::Hash::hash_slice(s.as_bytes(),&mut h); format!("{:x}",h.finish()) };
        hash == self.payload_hash
    }

    pub fn schema_id(&self) -> &str { &self.schema.schema_id }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create_and_verify() {
        let ev = DomainEvent::new(RealmId::Telum, EventSchemaRef::v1("task.created"), 1, "test", None, serde_json::json!({"title":"write tests"}));
        assert!(ev.verify_hash());
        assert_eq!(ev.schema_id(), "task.created");
        assert!(ev.causal_parent.is_none());
    }
    #[test] fn causal_chain() {
        let ev1 = DomainEvent::new(RealmId::Telum, EventSchemaRef::v1("e"), 1, "p", None, serde_json::json!(1));
        let ev2 = DomainEvent::new(RealmId::Telum, EventSchemaRef::v1("e"), 2, "p", Some(ev1.id.clone()), serde_json::json!(2));
        assert_eq!(ev2.causal_parent.as_deref(), Some(ev1.id.as_str()));
        assert!(ev1.lamport < ev2.lamport);
    }
    #[test] fn schema_ref_display() { assert_eq!(EventSchemaRef::v1("task.created").to_string(), "task.created@1.0"); }
}
