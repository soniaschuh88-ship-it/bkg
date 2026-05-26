use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ApprovalKind { Merge, DangerousToolUse, BudgetOverrun, AgentSpawn, SecretAccess, Custom }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ApprovalStatus { Pending, Approved, Rejected, Expired, Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String, pub kind: ApprovalKind, pub task_id: Option<String>,
    pub agent_id: Option<String>, pub description: String, pub context: serde_json::Value,
    pub status: ApprovalStatus, pub requested_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>, pub decided_by: Option<String>,
    pub rejection_reason: Option<String>,
}
impl ApprovalRequest {
    pub fn new(kind: ApprovalKind, desc: impl Into<String>) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), kind, task_id: None, agent_id: None, description: desc.into(), context: serde_json::Value::Null, status: ApprovalStatus::Pending, requested_at: Utc::now(), decided_at: None, decided_by: None, rejection_reason: None }
    }
    pub fn for_task(mut self, t: impl Into<String>) -> Self { self.task_id = Some(t.into()); self }
    pub fn is_pending(&self) -> bool { self.status == ApprovalStatus::Pending }
    pub fn is_decided(&self) -> bool { matches!(self.status, ApprovalStatus::Approved|ApprovalStatus::Rejected) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse { pub request_id: String, pub granted: bool, pub reason: Option<String>, pub decided_by: String, pub decided_at: DateTime<Utc> }
impl ApprovalResponse {
    pub fn grant(rid: impl Into<String>, by: impl Into<String>) -> Self { Self { request_id: rid.into(), granted: true, reason: None, decided_by: by.into(), decided_at: Utc::now() } }
    pub fn deny(rid: impl Into<String>, by: impl Into<String>, reason: impl Into<String>) -> Self { Self { request_id: rid.into(), granted: false, reason: Some(reason.into()), decided_by: by.into(), decided_at: Utc::now() } }
}
