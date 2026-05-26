use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::task::TaskId;
#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("dependency cycle detected involving task {0}")]
    Cycle(String),
    #[error("task {0} not found in graph")]
    NotFound(String),
}
/// Dependency graph for tasks. Ensures no cycles (DAG invariant).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph { edges: HashMap<String, HashSet<String>> }
impl DependencyGraph {
    pub fn new() -> Self { Self::default() }
    pub fn add_dependency(&mut self, task: &TaskId, depends_on: &TaskId) -> Result<(), DagError> {
        // Check for cycle before adding
        if self.would_create_cycle(&task.0, &depends_on.0) {
            return Err(DagError::Cycle(task.0.clone()));
        }
        self.edges.entry(task.0.clone()).or_default().insert(depends_on.0.clone());
        // ensure dependency node exists as a key (even if it's a leaf)
        self.edges.entry(depends_on.0.clone()).or_default();
        Ok(())
    }
    pub fn dependencies(&self, task: &TaskId) -> Vec<String> {
        self.edges.get(&task.0).cloned().unwrap_or_default().into_iter().collect()
    }
    pub fn is_ready(&self, task: &TaskId, done: &HashSet<String>) -> bool {
        self.dependencies(task).iter().all(|dep| done.contains(dep))
    }
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        if from == to { return true; }
        let mut visited = HashSet::new();
        self.reachable_from(to, &mut visited);
        visited.contains(from)
    }
    fn reachable_from(&self, start: &str, visited: &mut HashSet<String>) {
        if !visited.insert(start.to_string()) { return; }
        if let Some(deps) = self.edges.get(start) { for d in deps { self.reachable_from(d, visited); } }
    }
    pub fn topological_order(&self) -> Result<Vec<String>, DagError> {
        let mut result = vec![]; let mut visited = HashSet::new(); let mut in_stack = HashSet::new();
        for node in self.edges.keys() { if !visited.contains(node) { self.dfs(node, &mut visited, &mut in_stack, &mut result)?; } }
        Ok(result)
    }
    fn dfs(&self, node: &str, visited: &mut HashSet<String>, in_stack: &mut HashSet<String>, result: &mut Vec<String>) -> Result<(), DagError> {
        in_stack.insert(node.to_string()); visited.insert(node.to_string());
        if let Some(deps) = self.edges.get(node) { for dep in deps { if in_stack.contains(dep) { return Err(DagError::Cycle(dep.clone())); } else if !visited.contains(dep) { self.dfs(dep, visited, in_stack, result)?; } } }
        in_stack.remove(node); result.push(node.to_string()); Ok(())
    }
    pub fn node_count(&self) -> usize { self.edges.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn id(s: &str) -> TaskId { TaskId(s.to_string()) }
    #[test] fn simple_dep() { let mut g=DependencyGraph::new(); g.add_dependency(&id("T-2"),&id("T-1")).unwrap(); assert!(g.is_ready(&id("T-1"),&HashSet::new())); assert!(!g.is_ready(&id("T-2"),&HashSet::new())); }
    #[test] fn cycle_detected() { let mut g=DependencyGraph::new(); g.add_dependency(&id("T-2"),&id("T-1")).unwrap(); assert!(g.add_dependency(&id("T-1"),&id("T-2")).is_err()); }
    #[test] fn self_cycle() { let mut g=DependencyGraph::new(); assert!(g.add_dependency(&id("T-1"),&id("T-1")).is_err()); }
    #[test] fn topological() { let mut g=DependencyGraph::new(); g.add_dependency(&id("T-2"),&id("T-1")).unwrap(); g.add_dependency(&id("T-3"),&id("T-2")).unwrap(); let order=g.topological_order().unwrap(); let t1p=order.iter().position(|x|x=="T-1"); let t2p=order.iter().position(|x|x=="T-2"); let t3p=order.iter().position(|x|x=="T-3"); if let(Some(p1),Some(p2),Some(p3))=(t1p,t2p,t3p){assert!(p1<p2); assert!(p2<p3);}}
}