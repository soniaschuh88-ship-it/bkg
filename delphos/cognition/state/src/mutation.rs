// bkg-state/mutation.rs — typed mutation record with causality trace.
// Every state change is recorded here before being applied by the Reducer.
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_core::RealmId;

/// The kind of mutation applied to realm state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationKind {
    /// Entity created.
    Insert { entity_type: String, entity_id: String },
    /// Entity updated.
    Update { entity_type: String, entity_id: String, field: Option<String> },
    /// Entity removed.
    Delete { entity_type: String, entity_id: String },
    /// Metadata changed at the realm level.
    MetadataChange { key: String },
}

/// A single mutation record — the minimal "what changed" for audit + diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMutation {
    pub realm_id: RealmId,
    pub kind: MutationKind,
    pub caused_by_event_id: String,
    pub from_version: u64,
    pub to_version: u64,
    pub timestamp: DateTime<Utc>,
    /// Optional previous value for rollback support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_value: Option<serde_json::Value>,
    /// New value after mutation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_value: Option<serde_json::Value>,
}

impl StateMutation {
    pub fn insert(realm: RealmId, entity_type: &str, entity_id: &str, event_id: &str, from: u64, value: serde_json::Value) -> Self {
        Self {
            realm_id: realm,
            kind: MutationKind::Insert { entity_type: entity_type.into(), entity_id: entity_id.into() },
            caused_by_event_id: event_id.into(),
            from_version: from, to_version: from + 1,
            timestamp: Utc::now(),
            prev_value: None, new_value: Some(value),
        }
    }

    pub fn update(realm: RealmId, entity_type: &str, entity_id: &str, event_id: &str, from: u64, prev: serde_json::Value, next: serde_json::Value) -> Self {
        Self {
            realm_id: realm,
            kind: MutationKind::Update { entity_type: entity_type.into(), entity_id: entity_id.into(), field: None },
            caused_by_event_id: event_id.into(),
            from_version: from, to_version: from + 1,
            timestamp: Utc::now(),
            prev_value: Some(prev), new_value: Some(next),
        }
    }

    pub fn delete(realm: RealmId, entity_type: &str, entity_id: &str, event_id: &str, from: u64, prev: serde_json::Value) -> Self {
        Self {
            realm_id: realm,
            kind: MutationKind::Delete { entity_type: entity_type.into(), entity_id: entity_id.into() },
            caused_by_event_id: event_id.into(),
            from_version: from, to_version: from + 1,
            timestamp: Utc::now(),
            prev_value: Some(prev), new_value: None,
        }
    }

    pub fn is_reversible(&self) -> bool { self.prev_value.is_some() }
}

/// An ordered log of mutations for a single Reducer::apply() call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MutationLog(Vec<StateMutation>);

impl MutationLog {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, m: StateMutation) { self.0.push(m); }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn mutations(&self) -> &[StateMutation] { &self.0 }
    /// All entity types touched by this mutation batch.
    pub fn touched_entity_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.0.iter().filter_map(|m| match &m.kind {
            MutationKind::Insert { entity_type, .. } |
            MutationKind::Update { entity_type, .. } |
            MutationKind::Delete { entity_type, .. } => Some(entity_type.clone()),
            _ => None,
        }).collect();
        types.dedup();
        types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn insert_mutation() {
        let m = StateMutation::insert(RealmId::Telum, "task", "T-1", "evt-1", 0, serde_json::json!({"title":"x"}));
        assert!(!m.is_reversible()); // no prev_value
        assert_eq!(m.to_version, 1);
    }
    #[test] fn update_reversible() {
        let m = StateMutation::update(RealmId::Telum, "task", "T-1", "evt-2", 1, serde_json::json!({"status":"todo"}), serde_json::json!({"status":"in-progress"}));
        assert!(m.is_reversible());
    }
    #[test] fn mutation_log() {
        let mut log = MutationLog::new();
        log.push(StateMutation::insert(RealmId::Causa, "agent", "A-1", "e1", 0, serde_json::json!({})));
        log.push(StateMutation::insert(RealmId::Causa, "task", "T-1", "e1", 0, serde_json::json!({})));
        assert_eq!(log.len(), 2);
        assert_eq!(log.touched_entity_types().len(), 2);
    }
}
