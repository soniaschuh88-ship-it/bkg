// projection.rs — Projection trait: event → read model.
// UI reads ONLY from Projections. Never from the ledger directly.
use serde::{Deserialize, Serialize};
use crate::realm_state::RealmState;

/// Stable identifier for a Projection type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectionId(pub String);
impl ProjectionId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
}
impl std::fmt::Display for ProjectionId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "projection:{}", self.0) } }

/// A Projection is a disposable read model built from RealmState.
///
/// Projections are NEVER the source of truth.
/// If stale → discard and rebuild from the ledger via the Reducer.
pub trait Projection: Sized + Send + Sync {
    /// Unique stable id for this projection type.
    fn projection_id() -> ProjectionId;
    /// Build from current RealmState.
    fn build(state: &RealmState) -> Self;
    /// Deterministic checksum for staleness detection.
    fn checksum(&self) -> String;
    /// Whether this projection is stale given the current state checksum.
    fn is_stale(&self, state_checksum: &str) -> bool;
}

/// A simple key-value projection built from RealmState entities.
/// Used as the default projection for tests and simple UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityListProjection {
    pub realm_id: bkg_core::RealmId,
    pub entities: Vec<(String, serde_json::Value)>,
    pub state_checksum: String,
    pub version: u64,
}

impl Projection for EntityListProjection {
    fn projection_id() -> ProjectionId { ProjectionId::new("entity-list") }

    fn build(state: &crate::realm_state::RealmState) -> Self {
        Self {
            realm_id: state.realm_id,
            entities: state.entities.iter().map(|(k,v)| (k.clone(), v.clone())).collect(),
            state_checksum: state.checksum(),
            version: state.version,
        }
    }
    fn checksum(&self) -> String { self.state_checksum.clone() }
    fn is_stale(&self, state_checksum: &str) -> bool { self.state_checksum != state_checksum }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_state::RealmState;
    use bkg_core::RealmId;
    #[test] fn build_and_stale() {
        let mut s = RealmState::empty(RealmId::Telum);
        s.set_entity("task","T-1",serde_json::json!({"title":"x"}));
        let p = EntityListProjection::build(&s);
        assert!(!p.is_stale(&s.checksum()));
        s.set_entity("task","T-2",serde_json::json!({"title":"y"}));
        assert!(p.is_stale(&s.checksum())); // stale after mutation
    }
    #[test] fn disposable() {
        let s = RealmState::empty(RealmId::Styx);
        let p = EntityListProjection::build(&s);
        let p2 = EntityListProjection::build(&s);
        assert_eq!(p.checksum(), p2.checksum()); // deterministic rebuild
    }
}
