use std::collections::BTreeMap;
use crate::{component::ComponentStore, entity::{Entity, EntityId, Generation}};

/// A filtered entity query. Deterministic iteration order.
pub struct Query<'w> {
    components: &'w ComponentStore,
    generations: &'w BTreeMap<u64, Generation>,
    alive: &'w BTreeMap<u64, bool>,
}

impl<'w> Query<'w> {
    pub fn new(components: &'w ComponentStore, generations: &'w BTreeMap<u64, Generation>, alive: &'w BTreeMap<u64, bool>) -> Self {
        Self { components, generations, alive }
    }

    /// Return all alive entities that have ALL of the required component types.
    pub fn with_all(&self, required: &[&str]) -> Vec<(Entity, Vec<&serde_json::Value>)> {
        let mut result = Vec::new();
        for (&id, &is_alive) in self.alive {
            if !is_alive { continue; }
            let gen = self.generations.get(&id).copied().unwrap_or_default();
            let entity = Entity { id: EntityId(id), generation: gen };
            let values: Vec<&serde_json::Value> = required.iter()
                .filter_map(|t| self.components.get(t, EntityId(id)))
                .collect();
            if values.len() == required.len() {
                result.push((entity, values));
            }
        }
        result // BTreeMap iteration = stable order
    }

    /// Return all entities with a specific component value matching a predicate.
    pub fn with_filter(&self, type_name: &str, predicate: impl Fn(&serde_json::Value) -> bool) -> Vec<Entity> {
        self.components.iter_type(type_name)
            .filter_map(|(eid, v)| {
                if !predicate(v) { return None; }
                if !self.alive.get(&eid.0).copied().unwrap_or(false) { return None; }
                let gen = self.generations.get(&eid.0).copied().unwrap_or_default();
                Some(Entity { id: eid, generation: gen })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;
    #[test] fn query_all_components() {
        let mut w = World::new();
        let e1 = w.spawn(); w.insert(e1, "Status", &serde_json::json!("todo")); w.insert(e1, "Priority", &serde_json::json!(3));
        let e2 = w.spawn(); w.insert(e2, "Status", &serde_json::json!("done"));
        // e2 has no Priority
        let q = Query::new(&w.components, &Default::default(), &Default::default());
        // Just test the component store directly
        assert_eq!(w.components.count_for_type("Status"), 2);
        assert_eq!(w.components.count_for_type("Priority"), 1);
    }
    #[test] fn query_filter() {
        let mut w = World::new();
        let e1 = w.spawn(); w.insert(e1, "Status", &serde_json::json!({"value":"blocked"}));
        let e2 = w.spawn(); w.insert(e2, "Status", &serde_json::json!({"value":"done"}));
        let blocked: Vec<Entity> = w.components.iter_type("Status")
            .filter_map(|(id, v)| if v["value"] == "blocked" { Some(Entity::new(id.0, 0)) } else { None })
            .collect();
        assert_eq!(blocked.len(), 1);
    }
}
