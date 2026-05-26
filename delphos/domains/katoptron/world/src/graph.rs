use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RelationKind { DependsOn, BlockedBy, OwnedBy, PartOf, CausedBy, RelatesTo }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WorldEdge { pub from: String, pub to: String, pub kind: RelationKind, pub weight: f64 }
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct WorldGraph {
    pub nodes: BTreeSet<String>,
    pub edges: Vec<WorldEdge>,
    pub entity_types: BTreeMap<String, String>,
}
impl WorldGraph {
    pub fn new() -> Self { Self::default() }
    pub fn add_entity(&mut self, id: impl Into<String>, type_name: impl Into<String>) {
        let id = id.into();
        self.nodes.insert(id.clone());
        self.entity_types.insert(id, type_name.into());
    }
    pub fn add_relation(&mut self, from: impl Into<String>, to: impl Into<String>, kind: RelationKind, w: f64) {
        self.edges.push(WorldEdge { from: from.into(), to: to.into(), kind, weight: w });
    }
    pub fn entity_type(&self, id: &str) -> Option<&str> { self.entity_types.get(id).map(|s| s.as_str()) }
    pub fn relations_from(&self, id: &str) -> Vec<&WorldEdge> { self.edges.iter().filter(|e| e.from == id).collect() }
    pub fn entities_of_type(&self, t: &str) -> Vec<&str> {
        self.entity_types.iter().filter(|(_, v)| v.as_str() == t).map(|(k, _)| k.as_str()).collect()
    }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn build() {
        let mut g = WorldGraph::new();
        g.add_entity("T-1", "task"); g.add_entity("A-1", "agent");
        g.add_relation("A-1", "T-1", RelationKind::OwnedBy, 1.0);
        assert_eq!(g.node_count(), 2); assert_eq!(g.entities_of_type("task").len(), 1);
    }
}
