// event_abi.rs — typed event serialization contract.
// All events crossing realm or mesh boundaries are wrapped in EventAbiEnvelope.
use serde::{Deserialize, Serialize};
use crate::envelope::{AbiEnvelope, Symbol};

/// Wire format for a serialized domain event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAbiPayload {
    pub event_id: String,
    pub realm_id: String,
    pub schema_id: String,
    pub schema_version_major: u16,
    pub schema_version_minor: u16,
    pub lamport: u64,
    pub producer: String,
    pub causal_parent: Option<String>,
    pub payload: serde_json::Value,
    pub payload_hash: String,
}
pub type EventAbiEnvelope = AbiEnvelope<EventAbiPayload>;
impl EventAbiEnvelope {
    pub fn event_symbol() -> Symbol { Symbol::new("bkg.event.v1") }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn wrap_event() {
        let p = EventAbiPayload { event_id: "e1".into(), realm_id: "telum".into(), schema_id: "task.created".into(), schema_version_major: 1, schema_version_minor: 0, lamport: 1, producer: "test".into(), causal_parent: None, payload: serde_json::json!({"title":"x"}), payload_hash: "abc".into() };
        let env = AbiEnvelope::wrap(p, "bkg.event.v1").unwrap();
        assert!(env.verify_hash());
        assert_eq!(env.abi_version, AbiVersion::CURRENT);
    }
}