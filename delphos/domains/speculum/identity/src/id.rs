use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

#[derive(Debug, Clone, thiserror::Error)]
pub enum IdentityError {
    #[error("seed required for deterministic ID derivation")]
    NoSeed,
    #[error("invalid lineage: {0}")]
    InvalidLineage(String),
}

/// A deterministically derived ID.
/// DeterministicId::derive(seed, lineage, realm) always produces the same ID
/// for the same inputs — enabling reproducible replay.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeterministicId(pub String);

impl DeterministicId {
    /// Derive a deterministic ID from seed + lineage + realm.
    pub fn derive(seed: &str, lineage: &[&str], realm: RealmId) -> Self {
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        seed.hash(&mut h);
        for part in lineage { part.hash(&mut h); }
        realm.as_str().hash(&mut h);
        Self(format!("DID-{:016x}", std::hash::Hasher::finish(&h)))
    }

    /// Create a random (non-deterministic) ID for development use only.
    pub fn random() -> Self { Self(format!("DID-{}", uuid::Uuid::new_v4().as_simple())) }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for DeterministicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn deterministic() {
        let a = DeterministicId::derive("seed-1", &["task","T-001"], RealmId::Telum);
        let b = DeterministicId::derive("seed-1", &["task","T-001"], RealmId::Telum);
        assert_eq!(a, b);
    }
    #[test] fn different_seed_differs() {
        let a = DeterministicId::derive("seed-1", &["task"], RealmId::Telum);
        let b = DeterministicId::derive("seed-2", &["task"], RealmId::Telum);
        assert_ne!(a, b);
    }
    #[test] fn different_realm_differs() {
        let a = DeterministicId::derive("s", &[], RealmId::Telum);
        let b = DeterministicId::derive("s", &[], RealmId::Styx);
        assert_ne!(a, b);
    }
    #[test] fn display() { let id = DeterministicId::derive("x", &[], RealmId::Telum); assert!(id.to_string().starts_with("DID-")); }
}
