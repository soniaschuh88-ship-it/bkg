use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Fast lookup index: field_value → set of entity_ids.
/// Built over a projection's data for O(1) or O(log n) lookups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectionIndex {
    pub projection_id: String,
    pub indexed_field: String,
    /// value_string → BTreeSet<entity_id> (sorted = deterministic)
    index: BTreeMap<String, BTreeSet<String>>,
    pub built_at: DateTime<Utc>,
}

impl ProjectionIndex {
    pub fn build(projection_id: &str, field: &str, entities: &[serde_json::Value], id_field: &str) -> Self {
        let mut index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for entity in entities {
            if let (Some(id), Some(val)) = (entity.get(id_field), entity.get(field)) {
                let id_str = id.as_str().unwrap_or("").to_string();
                let val_str = val.as_str().map(String::from).unwrap_or_else(|| val.to_string());
                index.entry(val_str).or_default().insert(id_str);
            }
        }
        Self { projection_id: projection_id.into(), indexed_field: field.into(), index, built_at: chrono::Utc::now() }
    }

    pub fn lookup(&self, value: &str) -> Vec<&str> {
        self.index.get(value).map(|s| s.iter().map(|s| s.as_str()).collect()).unwrap_or_default()
    }

    pub fn cardinality(&self) -> usize { self.index.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entities() -> Vec<serde_json::Value> {
        vec![serde_json::json!({"id":"T-1","status":"blocked"}),serde_json::json!({"id":"T-2","status":"done"}),serde_json::json!({"id":"T-3","status":"blocked"})]
    }
    #[test] fn build_and_lookup() {
        let idx = ProjectionIndex::build("kanban","status",&entities(),"id");
        assert_eq!(idx.cardinality(), 2);
        let mut blocked = idx.lookup("blocked").to_vec(); blocked.sort();
        assert_eq!(blocked, vec!["T-1","T-3"]);
    }
    #[test] fn missing_value() { let idx = ProjectionIndex::build("k","status",&entities(),"id"); assert!(idx.lookup("nonexistent").is_empty()); }
}
