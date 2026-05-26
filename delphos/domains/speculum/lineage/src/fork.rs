use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ForkReason { Experiment, Recovery, Branching, Rollback }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ForkRecord { pub id: String, pub parent_snapshot_id: String, pub child_snapshot_id: String, pub reason: ForkReason, pub created_at: DateTime<Utc>, pub label: String }
impl ForkRecord {
    pub fn new(parent: impl Into<String>, child: impl Into<String>, reason: ForkReason, label: impl Into<String>) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), parent_snapshot_id: parent.into(), child_snapshot_id: child.into(), reason, created_at: Utc::now(), label: label.into() }
    }
}
