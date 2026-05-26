use serde::{Deserialize, Serialize};
use crate::graph::{RelationKind, WorldGraph};
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct WorldQuery { pub entity_type: Option<String>, pub limit: Option<usize> }
impl WorldQuery { pub fn of_type(t: impl Into<String>) -> Self { Self { entity_type: Some(t.into()), limit: None } } }
pub struct World { pub graph: WorldGraph, pub realm_id: String, pub version: u64 }
impl World {
    pub fn new(realm: impl Into<String>) -> Self {
        Self { graph: WorldGraph::new(), realm_id: realm.into(), version: 0 }
    }
    pub fn query(&self, q: &WorldQuery) -> Vec<String> {
        let all: Vec<String> = self.graph.nodes.iter().cloned().collect();
        let mut r = match &q.entity_type {
            Some(t) => all.into_iter().filter(|id| self.graph.entity_type(id) == Some(t.as_str())).collect(),
            None => all,
        };
        if let Some(lim) = q.limit { r.truncate(lim); }
        r
    }
    pub fn add_entity(&mut self, id: impl Into<String>, type_name: impl Into<String>) -> String {
        let id = id.into();
        self.graph.add_entity(&id, type_name);
        self.version += 1; id
    }
    pub fn add_relation(&mut self, from: impl Into<String>, to: impl Into<String>, kind: RelationKind) {
        self.graph.add_relation(from, to, kind, 1.0);
        self.version += 1;
    }
    pub fn entity_count(&self) -> usize { self.graph.node_count() }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn add_query() {
        let mut w = World::new("telum");
        w.add_entity("T-1", "task"); w.add_entity("A-1", "agent");
        assert_eq!(w.query(&WorldQuery::of_type("task")).len(), 1);
        assert_eq!(w.version, 2);
    }
}
