use serde::{Deserialize, Serialize};
use bkg_core::RealmId;
use crate::id::DeterministicId;

/// Stable identity for a DELPHOS realm instance.
/// Used for mesh node identification and fork tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmIdentity {
    pub id: DeterministicId,
    pub realm: RealmId,
    pub node_label: String,
    pub genesis_hash: String,
    pub fork_depth: u32,
    pub parent_id: Option<DeterministicId>,
}

impl RealmIdentity {
    pub fn genesis(realm: RealmId, genesis_hash: &str, node_label: &str) -> Self {
        let id = DeterministicId::derive(genesis_hash, &[node_label], realm);
        Self { id, realm, node_label: node_label.into(), genesis_hash: genesis_hash.into(), fork_depth: 0, parent_id: None }
    }
    pub fn fork(&self, new_label: &str) -> Self {
        let id = DeterministicId::derive(&self.genesis_hash, &[new_label, &self.fork_depth.to_string()], self.realm);
        Self { id, realm: self.realm, node_label: new_label.into(), genesis_hash: self.genesis_hash.clone(), fork_depth: self.fork_depth + 1, parent_id: Some(self.id.clone()) }
    }
    pub fn is_genesis(&self) -> bool { self.fork_depth == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn genesis_identity() {
        let ri = RealmIdentity::genesis(RealmId::Telum, "abc123", "node-0");
        assert!(ri.is_genesis());
        assert_eq!(ri.fork_depth, 0);
        assert!(ri.parent_id.is_none());
    }
    #[test] fn fork_creates_child() {
        let parent = RealmIdentity::genesis(RealmId::Telum, "abc", "n0");
        let child = parent.fork("n1");
        assert_eq!(child.fork_depth, 1);
        assert_eq!(child.parent_id.as_ref().unwrap().0, parent.id.0);
        assert!(!child.is_genesis());
    }
    #[test] fn deterministic_fork() {
        let p = RealmIdentity::genesis(RealmId::Causa, "seed", "root");
        let f1 = p.fork("branch");
        let f2 = p.fork("branch");
        assert_eq!(f1.id, f2.id); // same inputs = same ID
    }
}
