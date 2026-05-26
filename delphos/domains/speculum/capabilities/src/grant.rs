use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use crate::capability::CapabilitySet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum GrantStatus { Active, Expired, Revoked }

/// A time-bounded, revocable capability grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub id: String, pub grantee: String, pub capabilities: CapabilitySet,
    pub granted_by: String, pub status: GrantStatus,
    pub granted_at: DateTime<Utc>, pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>, pub revoke_reason: Option<String>,
}
impl CapabilityGrant {
    pub fn new(grantee: impl Into<String>, caps: CapabilitySet, grantor: impl Into<String>, ttl_secs: Option<i64>) -> Self {
        let now=Utc::now();
        Self { id: uuid::Uuid::new_v4().to_string(), grantee: grantee.into(), capabilities: caps, granted_by: grantor.into(), status: GrantStatus::Active, granted_at: now, expires_at: ttl_secs.map(|s| now+Duration::seconds(s)), revoked_at: None, revoke_reason: None }
    }
    pub fn is_active(&self) -> bool {
        if self.status != GrantStatus::Active { return false; }
        self.expires_at.map(|e| Utc::now() < e).unwrap_or(true)
    }
    pub fn has(&self, cap: &str) -> bool { self.is_active() && self.capabilities.has(cap) }
    pub fn revoke(&mut self, reason: impl Into<String>) { self.status=GrantStatus::Revoked; self.revoked_at=Some(Utc::now()); self.revoke_reason=Some(reason.into()); }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityId, CapabilitySet};
    #[test] fn active_grant() { let g=CapabilityGrant::new("agent-1",CapabilitySet::workspace_write(),"operator",Some(3600)); assert!(g.is_active()); assert!(g.has(CapabilityId::EXECUTE_BASH)); }
    #[test] fn revoke() { let mut g=CapabilityGrant::new("agent-1",CapabilitySet::read_only(),"op",None); g.revoke("policy violation"); assert!(!g.is_active()); assert_eq!(g.status,GrantStatus::Revoked); }
    #[test] fn expired_ttl_0() { let g=CapabilityGrant::new("a",CapabilitySet::new(),"op",Some(-1)); assert!(!g.is_active()); }
}
