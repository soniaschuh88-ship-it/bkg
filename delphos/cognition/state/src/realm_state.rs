// realm_state.rs — RealmState: immutable, versioned snapshot of one realm.
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;
use bkg_clock::tick::SequencedInstant;

/// The reconstructed state of a single DELPHOS realm.
/// Immutable: mutation produces a NEW RealmState (copy-on-write).
/// Never stored as the source of truth — always rebuildable from the event ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealmState {
    pub realm_id: RealmId,
    /// Monotonically increasing version counter.
    pub version: u64,
    /// Lamport clock at the time of this state.
    pub timestamp: Option<SequencedInstant>,
    /// Hash of the event that produced this state (for chaining).
    pub event_hash: Option<String>,
    /// Typed entity data keyed by (entity_type, entity_id).
    pub entities: BTreeMap<String, serde_json::Value>,
    /// Arbitrary realm-level metadata.
    pub metadata: serde_json::Value,
}

impl RealmState {
    pub fn empty(realm_id: RealmId) -> Self {
        Self { realm_id, version: 0, timestamp: None, event_hash: None, entities: BTreeMap::new(), metadata: serde_json::Value::Null }
    }

    /// Produce the next version of this state (copy-on-write).
    pub fn next_version(mut self, timestamp: Option<SequencedInstant>, event_hash: Option<String>) -> Self {
        self.version += 1;
        self.timestamp = timestamp;
        self.event_hash = event_hash;
        self
    }

    pub fn set_entity(&mut self, entity_type: &str, entity_id: &str, value: serde_json::Value) {
        self.entities.insert(format!("{entity_type}/{entity_id}"), value);
    }

    pub fn get_entity(&self, entity_type: &str, entity_id: &str) -> Option<&serde_json::Value> {
        self.entities.get(&format!("{entity_type}/{entity_id}"))
    }

    pub fn remove_entity(&mut self, entity_type: &str, entity_id: &str) -> Option<serde_json::Value> {
        self.entities.remove(&format!("{entity_type}/{entity_id}"))
    }

    pub fn entity_count(&self) -> usize { self.entities.len() }

    /// Deterministic checksum over entities for drift detection.
    pub fn checksum(&self) -> String {
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for (k, v) in &self.entities {
            k.hash(&mut h);
            v.to_string().hash(&mut h);
        }
        format!("{:x}", std::hash::Hasher::finish(&h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty() { let s=RealmState::empty(RealmId::Telum); assert_eq!(s.version,0); assert_eq!(s.entity_count(),0); }
    #[test] fn version_bump() { let s=RealmState::empty(RealmId::Telum).next_version(None,None); assert_eq!(s.version,1); }
    #[test] fn entities() {
        let mut s=RealmState::empty(RealmId::Telum);
        s.set_entity("task","T-1",serde_json::json!({"title":"write bkg-state"}));
        assert!(s.get_entity("task","T-1").is_some());
        assert_eq!(s.entity_count(),1);
        s.remove_entity("task","T-1");
        assert_eq!(s.entity_count(),0);
    }
    #[test] fn checksum_stable() {
        let mut s=RealmState::empty(RealmId::Causa);
        s.set_entity("x","1",serde_json::json!(42));
        let c1=s.checksum(); let c2=s.checksum();
        assert_eq!(c1,c2);
    }
    #[test] fn checksum_changes() {
        let mut s=RealmState::empty(RealmId::Causa);
        s.set_entity("x","1",serde_json::json!(42));
        let c1=s.checksum();
        s.set_entity("x","1",serde_json::json!(43));
        assert_ne!(c1,s.checksum());
    }
}
