use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use crate::fork::ForkRecord;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LineageEdge { pub from: String, pub to: String, pub fork_id: String }
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct LineageGraph { edges: Vec<LineageEdge>, nodes: BTreeSet<String> }
impl LineageGraph {
    pub fn new() -> Self { Self::default() }
    pub fn record_fork(&mut self, f: &ForkRecord) {
        self.nodes.insert(f.parent_snapshot_id.clone());
        self.nodes.insert(f.child_snapshot_id.clone());
        self.edges.push(LineageEdge { from: f.parent_snapshot_id.clone(), to: f.child_snapshot_id.clone(), fork_id: f.id.clone() });
    }
    pub fn ancestors_of(&self, id: &str) -> Vec<String> { self.edges.iter().filter(|e| e.to == id).map(|e| e.from.clone()).collect() }
    pub fn descendants_of(&self, id: &str) -> Vec<String> { self.edges.iter().filter(|e| e.from == id).map(|e| e.to.clone()).collect() }
    pub fn node_count(&self) -> usize { self.nodes.len() }
}
#[cfg(test)]
mod tests { use super::*; use crate::fork::{ForkRecord, ForkReason};
    #[test] fn fork_query() {
        let mut g = LineageGraph::new();
        g.record_fork(&ForkRecord::new("s1", "s2", ForkReason::Branching, "b"));
        assert_eq!(g.ancestors_of("s2"), vec!["s1"]);
        assert_eq!(g.node_count(), 2);
    }
}
