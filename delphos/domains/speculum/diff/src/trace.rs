use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_core::RealmId;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CausalTrace { pub event_id: String, pub caused_by: Option<String>, pub realm: RealmId, pub entity_type: String, pub entity_id: String, pub mutation_kind: String, pub actor: String, pub timestamp: DateTime<Utc> }
impl CausalTrace {
    pub fn new(event_id: impl Into<String>, realm: RealmId, etype: impl Into<String>, eid: impl Into<String>, kind: impl Into<String>, actor: impl Into<String>) -> Self {
        Self { event_id: event_id.into(), caused_by: None, realm, entity_type: etype.into(), entity_id: eid.into(), mutation_kind: kind.into(), actor: actor.into(), timestamp: Utc::now() }
    }
    pub fn with_parent(mut self, p: impl Into<String>) -> Self { self.caused_by = Some(p.into()); self }
}
