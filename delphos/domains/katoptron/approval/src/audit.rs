use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
/// Append-only audit event for approval decisions. Single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent { pub id: String, pub request_id: String, pub kind: String, pub action: String, pub actor: String, pub detail: Option<String>, pub timestamp: DateTime<Utc> }
impl AuditEvent {
    pub fn approved(req_id: &str, kind: &str, actor: &str) -> Self { Self { id: uuid::Uuid::new_v4().to_string(), request_id: req_id.into(), kind: kind.into(), action: "approved".into(), actor: actor.into(), detail: None, timestamp: Utc::now() } }
    pub fn rejected(req_id: &str, kind: &str, actor: &str, reason: &str) -> Self { Self { id: uuid::Uuid::new_v4().to_string(), request_id: req_id.into(), kind: kind.into(), action: "rejected".into(), actor: actor.into(), detail: Some(reason.into()), timestamp: Utc::now() } }
}
#[derive(Debug, Default)]
pub struct ApprovalAudit { events: Vec<AuditEvent> }
impl ApprovalAudit {
    pub fn new() -> Self { Self::default() }
    pub fn record(&mut self, e: AuditEvent) { self.events.push(e); }
    pub fn events_for(&self, request_id: &str) -> Vec<&AuditEvent> { self.events.iter().filter(|e| e.request_id==request_id).collect() }
    pub fn count(&self) -> usize { self.events.len() }
}
