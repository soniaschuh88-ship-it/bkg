use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use bkg_core::{BkgError, BkgResult};
use crate::request::{ApprovalRequest, ApprovalResponse, ApprovalStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum GateStatus { #[default] Open, AwaitingApproval, Approved, Rejected }

#[derive(Debug, Default)]
pub struct ApprovalGate { requests: HashMap<String, ApprovalRequest> }
impl ApprovalGate {
    pub fn new() -> Self { Self::default() }
    pub fn submit(&mut self, req: ApprovalRequest) -> String { let id = req.id.clone(); self.requests.insert(id.clone(), req); id }
    pub fn decide(&mut self, resp: &ApprovalResponse) -> BkgResult<()> {
        let req = self.requests.get_mut(&resp.request_id).ok_or_else(|| BkgError::Internal(format!("approval {} not found",resp.request_id)))?;
        if !req.is_pending() { return Err(BkgError::Internal("already decided".into())); }
        req.status = if resp.granted { ApprovalStatus::Approved } else { ApprovalStatus::Rejected };
        req.decided_at = Some(chrono::Utc::now()); req.decided_by = Some(resp.decided_by.clone());
        if !resp.granted { req.rejection_reason = resp.reason.clone(); }
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&ApprovalRequest> { self.requests.get(id) }
    pub fn pending(&self) -> Vec<&ApprovalRequest> { self.requests.values().filter(|r| r.is_pending()).collect() }
    pub fn count(&self) -> usize { self.requests.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{ApprovalKind, ApprovalRequest, ApprovalResponse};
    #[test] fn submit_approve() {
        let mut g=ApprovalGate::new();
        let id=g.submit(ApprovalRequest::new(ApprovalKind::Merge,"merge T-1"));
        assert_eq!(g.pending().len(),1);
        g.decide(&ApprovalResponse::grant(&id,"operator")).unwrap();
        assert_eq!(g.pending().len(),0);
        assert!(g.get(&id).unwrap().is_decided());
    }
    #[test] fn double_decide_fails() {
        let mut g=ApprovalGate::new();
        let id=g.submit(ApprovalRequest::new(ApprovalKind::Custom,"x"));
        g.decide(&ApprovalResponse::grant(&id,"op")).unwrap();
        assert!(g.decide(&ApprovalResponse::grant(&id,"op")).is_err());
    }
    #[test] fn reject_has_reason() {
        let mut g=ApprovalGate::new();
        let id=g.submit(ApprovalRequest::new(ApprovalKind::DangerousToolUse,"rm -rf /"));
        g.decide(&ApprovalResponse::deny(&id,"safety","too dangerous")).unwrap();
        assert_eq!(g.get(&id).unwrap().rejection_reason.as_deref(),Some("too dangerous"));
    }
}
