use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::entity::EntityId;

/// Marker trait for all ECS components.
pub trait Component: Send + Sync + 'static {}

/// Type-erased component storage using JSON. Deterministic BTreeMap order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentStore {
    /// component_type_name → entity_id → json value
    stores: BTreeMap<String, BTreeMap<u64, serde_json::Value>>,
}

impl ComponentStore {
    pub fn new() -> Self { Self::default() }

    pub fn insert(&mut self, type_name: &str, entity_id: EntityId, value: serde_json::Value) {
        self.stores.entry(type_name.to_string()).or_default().insert(entity_id.0, value);
    }

    pub fn get(&self, type_name: &str, entity_id: EntityId) -> Option<&serde_json::Value> {
        self.stores.get(type_name)?.get(&entity_id.0)
    }

    pub fn remove(&mut self, type_name: &str, entity_id: EntityId) -> Option<serde_json::Value> {
        self.stores.get_mut(type_name)?.remove(&entity_id.0)
    }

    /// Iterate all entities with a given component. Stable order (BTreeMap).
    pub fn iter_type(&self, type_name: &str) -> impl Iterator<Item=(EntityId, &serde_json::Value)> {
        self.stores.get(type_name).into_iter()
            .flat_map(|m| m.iter().map(|(k, v)| (EntityId(*k), v)))
    }

    pub fn component_types(&self) -> Vec<&str> { self.stores.keys().map(|s| s.as_str()).collect() }
    pub fn count_for_type(&self, type_name: &str) -> usize { self.stores.get(type_name).map(|m| m.len()).unwrap_or(0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn insert_get() {
        let mut s = ComponentStore::new();
        s.insert("TaskStatus", EntityId(1), serde_json::json!({"status":"todo"}));
        assert!(s.get("TaskStatus", EntityId(1)).is_some());
        assert!(s.get("TaskStatus", EntityId(99)).is_none());
    }
    #[test] fn stable_iteration() {
        let mut s = ComponentStore::new();
        for i in [3u64,1,4,2] { s.insert("Pos", EntityId(i), serde_json::json!({"x":i})); }
        let ids: Vec<u64> = s.iter_type("Pos").map(|(e,_)| e.0).collect();
        assert_eq!(ids, vec![1,2,3,4]); // BTreeMap order
    }
    #[test] fn remove() {
        let mut s = ComponentStore::new();
        s.insert("T", EntityId(1), serde_json::json!({}));
        assert!(s.remove("T", EntityId(1)).is_some());
        assert!(s.get("T", EntityId(1)).is_none());
    }
}
