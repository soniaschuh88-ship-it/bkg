use serde::{Deserialize,Serialize};
use bkg_core::{AgentId,TaskId};
use bkg_runtime::TaskPayload;
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum TaskPriority{Low=1,Normal=5,High=10,Critical=100}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum TaskStatus{Pending,Ready,Running,Completed,Failed,Cancelled}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Task{pub id:TaskId,pub label:String,pub payload:TaskPayload,pub priority:TaskPriority,pub status:TaskStatus,#[serde(default,skip_serializing_if="Option::is_none")]pub assigned_agent:Option<AgentId>,pub created_at:chrono::DateTime<chrono::Utc>,#[serde(default,skip_serializing_if="Option::is_none")]pub completed_at:Option<chrono::DateTime<chrono::Utc>>}
impl Task{
    pub fn new(l:impl Into<String>,p:TaskPayload,pri:TaskPriority)->Self{Self{id:TaskId::new(),label:l.into(),payload:p,priority:pri,status:TaskStatus::Pending,assigned_agent:None,created_at:chrono::Utc::now(),completed_at:None}}
    pub fn is_terminal(&self)->bool{matches!(self.status,TaskStatus::Completed|TaskStatus::Failed|TaskStatus::Cancelled)}
}
