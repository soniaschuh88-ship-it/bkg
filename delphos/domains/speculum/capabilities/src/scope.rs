use serde::{Deserialize, Serialize};
use crate::capability::CapabilitySet;

/// A signed execution scope — passed into sandboxed tool invocations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionScope {
    pub scope_id: String, pub agent_id: String, pub task_id: Option<String>,
    pub capabilities: CapabilitySet, pub workspace_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
impl ExecutionScope {
    pub fn new(agent_id: impl Into<String>, caps: CapabilitySet) -> Self {
        Self { scope_id: uuid::Uuid::new_v4().to_string(), agent_id: agent_id.into(), task_id: None, capabilities: caps, workspace_path: None, created_at: chrono::Utc::now() }
    }
    pub fn for_task(mut self, t: impl Into<String>) -> Self { self.task_id=Some(t.into()); self }
    pub fn with_workspace(mut self, p: impl Into<String>) -> Self { self.workspace_path=Some(p.into()); self }
    pub fn can(&self, cap: &str) -> bool { self.capabilities.has(cap) }
}
