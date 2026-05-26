use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CapabilityId(pub String);
impl CapabilityId {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    // Well-known capability IDs
    pub const READ_FILES: &'static str = "files:read";
    pub const WRITE_FILES: &'static str = "files:write";
    pub const EXECUTE_BASH: &'static str = "bash:execute";
    pub const NETWORK_OUT: &'static str = "network:outbound";
    pub const SPAWN_AGENT: &'static str = "agent:spawn";
    pub const ACCESS_SECRETS: &'static str = "secrets:access";
}
impl std::fmt::Display for CapabilityId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }

/// An ordered set of capability IDs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet(BTreeSet<String>);
impl CapabilitySet {
    pub fn new() -> Self { Self::default() }
    pub fn read_only() -> Self { let mut s=Self::new(); s.add(CapabilityId::READ_FILES); s }
    pub fn workspace_write() -> Self { let mut s=Self::read_only(); s.add(CapabilityId::WRITE_FILES); s.add(CapabilityId::EXECUTE_BASH); s }
    pub fn add(&mut self, cap: impl AsRef<str>) { self.0.insert(cap.as_ref().to_string()); }
    pub fn remove(&mut self, cap: &str) { self.0.remove(cap); }
    pub fn has(&self, cap: &str) -> bool { self.0.contains(cap) }
    pub fn iter(&self) -> impl Iterator<Item=&str> { self.0.iter().map(|s| s.as_str()) }
    pub fn count(&self) -> usize { self.0.len() }
    pub fn is_subset_of(&self, other: &CapabilitySet) -> bool { self.0.iter().all(|c| other.0.contains(c)) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn read_only() { let s=CapabilitySet::read_only(); assert!(s.has(CapabilityId::READ_FILES)); assert!(!s.has(CapabilityId::WRITE_FILES)); }
    #[test] fn workspace_write_includes_read() { let s=CapabilitySet::workspace_write(); assert!(s.has(CapabilityId::READ_FILES)); assert!(s.has(CapabilityId::EXECUTE_BASH)); }
    #[test] fn subset() { let ro=CapabilitySet::read_only(); let rw=CapabilitySet::workspace_write(); assert!(ro.is_subset_of(&rw)); assert!(!rw.is_subset_of(&ro)); }
}
