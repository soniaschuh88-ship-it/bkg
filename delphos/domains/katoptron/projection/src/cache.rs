use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// One cached projection entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionEntry {
    pub projection_id: String,
    pub realm_id: String,
    pub state_version: u64,
    pub state_checksum: String,
    pub data: serde_json::Value,
    pub built_at: DateTime<Utc>,
    pub build_count: u64,
}

impl ProjectionEntry {
    pub fn is_stale(&self, current_checksum: &str) -> bool {
        self.state_checksum != current_checksum
    }
}

/// In-memory projection cache. Indexed by (realm_id, projection_id).
/// Disposable: cleared on state checksum mismatch. Rebuildable from ledger.
#[derive(Debug, Default)]
pub struct ProjectionCache {
    entries: BTreeMap<String, ProjectionEntry>,
}

impl ProjectionCache {
    pub fn new() -> Self { Self::default() }

    fn key(realm_id: &str, projection_id: &str) -> String {
        format!("{realm_id}/{projection_id}")
    }

    pub fn insert(&mut self, entry: ProjectionEntry) {
        self.entries.insert(Self::key(&entry.realm_id, &entry.projection_id), entry);
    }

    pub fn get(&self, realm_id: &str, projection_id: &str) -> Option<&ProjectionEntry> {
        self.entries.get(&Self::key(realm_id, projection_id))
    }

    pub fn is_stale(&self, realm_id: &str, projection_id: &str, current_checksum: &str) -> bool {
        match self.get(realm_id, projection_id) {
            None => true,
            Some(e) => e.is_stale(current_checksum),
        }
    }

    /// Invalidate all projections for a realm (called when state changes).
    pub fn invalidate_realm(&mut self, realm_id: &str) {
        self.entries.retain(|k, _| !k.starts_with(realm_id));
    }

    /// Invalidate one specific projection.
    pub fn invalidate(&mut self, realm_id: &str, projection_id: &str) {
        self.entries.remove(&Self::key(realm_id, projection_id));
    }

    pub fn count(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    fn entry(rid: &str, pid: &str, cksum: &str) -> ProjectionEntry {
        ProjectionEntry { projection_id: pid.into(), realm_id: rid.into(), state_version: 1, state_checksum: cksum.into(), data: serde_json::json!({}), built_at: Utc::now(), build_count: 1 }
    }
    #[test] fn insert_get() { let mut c=ProjectionCache::new(); c.insert(entry("telum","kanban","abc")); assert!(c.get("telum","kanban").is_some()); }
    #[test] fn stale_detection() { let mut c=ProjectionCache::new(); c.insert(entry("telum","kanban","abc")); assert!(!c.is_stale("telum","kanban","abc")); assert!(c.is_stale("telum","kanban","xyz")); }
    #[test] fn missing_is_stale() { let c=ProjectionCache::new(); assert!(c.is_stale("x","y","z")); }
    #[test] fn invalidate_realm() { let mut c=ProjectionCache::new(); c.insert(entry("telum","kanban","a")); c.insert(entry("telum","tasks","b")); c.insert(entry("styx","events","c")); c.invalidate_realm("telum"); assert_eq!(c.count(),1); }
    #[test] fn invalidate_one() { let mut c=ProjectionCache::new(); c.insert(entry("r","p1","a")); c.insert(entry("r","p2","b")); c.invalidate("r","p1"); assert!(c.get("r","p1").is_none()); assert!(c.get("r","p2").is_some()); }
}
