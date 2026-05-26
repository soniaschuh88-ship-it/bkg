use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::{component::ComponentStore, entity::{Entity, EntityId, Generation}};

/// The ECS World — entity registry + component stores.
/// Deterministic: BTreeMap ordering, stable entity IDs, generation counters.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct World {
    /// Next entity ID to allocate (monotonically increasing).
    next_id: u64,
    /// Generation per entity slot.
    generations: BTreeMap<u64, Generation>,
    /// Whether each entity is alive.
    alive: BTreeMap<u64, bool>,
    /// All component data.
    pub components: ComponentStore,
    /// Entity creation order (for stable iteration).
    creation_order: Vec<u64>,
}

impl World {
    pub fn new() -> Self { Self::default() }

    /// Spawn a new entity. Returns a stable Entity handle.
    pub fn spawn(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        let gen = Generation(0);
        self.generations.insert(id, gen);
        self.alive.insert(id, true);
        self.creation_order.push(id);
        Entity { id: EntityId(id), generation: gen }
    }

    /// Despawn an entity, incrementing its generation.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) { return false; }
        let gen = self.generations.entry(entity.id.0).or_default();
        *gen = gen.next();
        self.alive.insert(entity.id.0, false);
        let types: Vec<String> = self.components.component_types().iter().map(|s| s.to_string()).collect();
        for t in types { self.components.remove(&t, entity.id); }
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.generations.get(&entity.id.0).copied() == Some(entity.generation)
            && self.alive.get(&entity.id.0).copied().unwrap_or(false)
    }

    /// Iterate alive entities in creation order (deterministic).
    pub fn alive_entities(&self) -> impl Iterator<Item=Entity> + '_ {
        self.creation_order.iter().filter_map(move |&id| {
            if self.alive.get(&id).copied().unwrap_or(false) {
                let gen = self.generations.get(&id).copied().unwrap_or_default();
                Some(Entity { id: EntityId(id), generation: gen })
            } else { None }
        })
    }

    pub fn entity_count(&self) -> usize { self.alive.values().filter(|&&a| a).count() }
    pub fn total_spawned(&self) -> u64 { self.next_id }

    /// Insert a component for an entity.
    pub fn insert<T: serde::Serialize>(&mut self, entity: Entity, type_name: &str, component: &T) {
        if !self.is_alive(entity) { return; }
        self.components.insert(type_name, entity.id, serde_json::to_value(component).unwrap_or_default());
    }

    /// Get a component value for an entity.
    pub fn get(&self, entity: Entity, type_name: &str) -> Option<&serde_json::Value> {
        if !self.is_alive(entity) { return None; }
        self.components.get(type_name, entity.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn spawn_and_count() {
        let mut w = World::new();
        let e1 = w.spawn(); let _e2 = w.spawn();
        assert_eq!(w.entity_count(), 2);
        assert!(w.is_alive(e1));
    }
    #[test] fn despawn() {
        let mut w = World::new();
        let e = w.spawn();
        assert!(w.despawn(e));
        assert!(!w.is_alive(e));
        assert_eq!(w.entity_count(), 0);
    }
    #[test] fn stale_handle() {
        let mut w = World::new();
        let e = w.spawn(); let old = e;
        w.despawn(e);
        let _e2 = w.spawn(); // reuse slot conceptually
        assert!(!w.is_alive(old)); // old generation is dead
    }
    #[test] fn insert_get_component() {
        let mut w = World::new();
        let e = w.spawn();
        w.insert(e, "Status", &serde_json::json!({"value":"todo"}));
        assert!(w.get(e, "Status").is_some());
        assert!(w.get(e, "Missing").is_none());
    }
    #[test] fn creation_order_stable() {
        let mut w = World::new();
        let ids: Vec<u64> = (0..5).map(|_| w.spawn().id.0).collect();
        let iter_ids: Vec<u64> = w.alive_entities().map(|e| e.id.0).collect();
        assert_eq!(ids, iter_ids);
    }
}
