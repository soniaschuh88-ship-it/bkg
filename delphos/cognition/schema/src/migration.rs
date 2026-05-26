// migration.rs — replay-safe schema migration engine.
// When a schema version changes, old events must be transformed before the Reducer sees them.
use crate::schema::{MigrationStrategy, SchemaVersion};

/// Result of applying a migration to a raw event payload.
#[derive(Debug, Clone)]
pub enum MigrationResult {
    /// Payload transformed successfully.
    Transformed(serde_json::Value),
    /// No transformation needed.
    Passthrough(serde_json::Value),
    /// Event should be skipped entirely.
    Skipped,
    /// Migration rejected — replay must stop.
    Rejected(String),
}

impl MigrationResult {
    pub fn is_fatal(&self) -> bool { matches!(self, Self::Rejected(_)) }
    pub fn payload(self) -> Option<serde_json::Value> {
        match self { Self::Transformed(v)|Self::Passthrough(v) => Some(v), _ => None }
    }
}

/// Apply a migration strategy to an event payload.
pub fn apply_migration(
    payload: serde_json::Value,
    stored_version: SchemaVersion,
    current_version: SchemaVersion,
    strategy: &MigrationStrategy,
) -> MigrationResult {
    if stored_version == current_version { return MigrationResult::Passthrough(payload); }
    match strategy {
        MigrationStrategy::Passthrough => MigrationResult::Passthrough(payload),
        MigrationStrategy::Skip => MigrationResult::Skipped,
        MigrationStrategy::Reject { reason } => MigrationResult::Rejected(reason.clone()),
        MigrationStrategy::Transform { transformer_id } => {
            // In a full implementation: look up and run the transformer by ID.
            // For now: passthrough with a version annotation added.
            let mut p = payload;
            if let Some(obj) = p.as_object_mut() {
                obj.insert("_migrated_from".into(), serde_json::json!(stored_version.to_string()));
                obj.insert("_transformer".into(), serde_json::json!(transformer_id));
            }
            MigrationResult::Transformed(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn same_version_passthrough() {
        let r = apply_migration(serde_json::json!({"x":1}), SchemaVersion::V1, SchemaVersion::V1, &MigrationStrategy::Passthrough);
        assert!(matches!(r, MigrationResult::Passthrough(_)));
    }
    #[test] fn skip_strategy() {
        let r = apply_migration(serde_json::json!({}), SchemaVersion::new(1,0), SchemaVersion::new(1,1), &MigrationStrategy::Skip);
        assert!(matches!(r, MigrationResult::Skipped));
    }
    #[test] fn reject_strategy() {
        let r = apply_migration(serde_json::json!({}), SchemaVersion::new(1,0), SchemaVersion::new(1,1), &MigrationStrategy::Reject { reason: "deprecated".into() });
        assert!(r.is_fatal());
    }
    #[test] fn transform_adds_metadata() {
        let r = apply_migration(serde_json::json!({"title":"x"}), SchemaVersion::new(1,0), SchemaVersion::new(1,1), &MigrationStrategy::Transform { transformer_id: "t1".into() });
        let payload = r.payload().unwrap();
        assert!(payload.get("_migrated_from").is_some());
    }
}