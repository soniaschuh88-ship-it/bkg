use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::id::DeterministicId;
use bkg_core::RealmId;

/// One node in a lineage (ancestry) chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub id: DeterministicId,
    pub realm: RealmId,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<DeterministicId>,
}

impl LineageNode {
    pub fn root(id: DeterministicId, realm: RealmId, label: impl Into<String>) -> Self {
        Self { id, realm, label: label.into(), created_at: Utc::now(), parent_id: None }
    }
    pub fn child(id: DeterministicId, realm: RealmId, label: impl Into<String>, parent: DeterministicId) -> Self {
        Self { id, realm, label: label.into(), created_at: Utc::now(), parent_id: Some(parent) }
    }
    pub fn is_root(&self) -> bool { self.parent_id.is_none() }
}

/// A complete ancestry chain: ordered list of LineageNodes from root to leaf.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AncestryChain { nodes: Vec<LineageNode> }

impl AncestryChain {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, node: LineageNode) { self.nodes.push(node); }
    pub fn depth(&self) -> usize { self.nodes.len() }
    pub fn root(&self) -> Option<&LineageNode> { self.nodes.first() }
    pub fn leaf(&self) -> Option<&LineageNode> { self.nodes.last() }
    pub fn common_ancestor(&self, other: &AncestryChain) -> Option<&LineageNode> {
        self.nodes.iter().find(|node| other.nodes.iter().any(|n| n.id == node.id))
    }
    pub fn ids(&self) -> Vec<&DeterministicId> { self.nodes.iter().map(|n| &n.id).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_chain() -> AncestryChain {
        let root_id = DeterministicId::derive("genesis",&[],RealmId::Telum);
        let child_id = DeterministicId::derive("genesis",&["child"],RealmId::Telum);
        let mut c = AncestryChain::new();
        c.push(LineageNode::root(root_id.clone()  ,RealmId::Telum,"genesis"));
        c.push(LineageNode::child(child_id,RealmId::Telum,"branch-1",root_id));
        c
    }
    // Workaround: DeterministicId::derive doesn't take RealmId as 3rd arg in old sig
    // Quick fix: inline the calls with the right sig
    #[test] fn depth() { assert_eq!(make_chain().depth(), 2); }
    #[test] fn root_and_leaf() { let c=make_chain(); assert!(c.root().unwrap().is_root()); assert!(!c.leaf().unwrap().is_root()); }
}
