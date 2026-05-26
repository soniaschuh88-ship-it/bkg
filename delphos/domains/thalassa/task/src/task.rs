use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Stable task identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);
impl TaskId { pub fn new() -> Self { Self(format!("T-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase())) } }
impl Default for TaskId { fn default() -> Self { Self::new() } }
impl std::fmt::Display for TaskId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum TaskPriority { Low, #[default] Normal, High, Urgent }
impl TaskPriority { pub fn as_str(self) -> &'static str { match self { Self::Low=>"low", Self::Normal=>"normal", Self::High=>"high", Self::Urgent=>"urgent" } } }

/// A BKG task — the primary unit of agent work.
/// State is always reconstructed from events (bkg-state invariant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: crate::lifecycle::TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<String>,
    pub prompt_md: Option<String>,
    pub branch: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub tags: Vec<String>,
    /// bkg-agents AgentId as string
    pub agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl Task {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self { id: TaskId::new(), title: title.into(), description: None, status: crate::lifecycle::TaskStatus::Planning, priority: TaskPriority::default(), assignee: None, prompt_md: None, branch: None, acceptance_criteria: vec![], tags: vec![], agent_id: None, created_at: now, updated_at: now }
    }
    pub fn with_priority(mut self, p: TaskPriority) -> Self { self.priority = p; self }
    pub fn with_agent(mut self, agent: impl Into<String>) -> Self { self.agent_id = Some(agent.into()); self }
    pub fn is_terminal(&self) -> bool { self.status.is_terminal() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create() { let t=Task::new("write bkg-task"); assert!(t.id.0.starts_with("T-")); assert!(!t.is_terminal()); }
    #[test] fn priority_order() { assert!(TaskPriority::Urgent > TaskPriority::Low); }
}