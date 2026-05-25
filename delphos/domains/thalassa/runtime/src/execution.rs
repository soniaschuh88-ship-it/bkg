use serde::{Deserialize,Serialize};
use bkg_core::{AgentId,ExecutionSeed,RealmId,SessionId};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TaskPayload{pub label:String,pub input:serde_json::Value,pub seed:ExecutionSeed,#[serde(default,skip_serializing_if="Option::is_none")]pub target_realm:Option<RealmId>}
impl TaskPayload{
    pub fn new(l:impl Into<String>,i:serde_json::Value)->Self{Self{label:l.into(),input:i,seed:ExecutionSeed::random(),target_realm:None}}
    pub fn with_seed(mut self,s:ExecutionSeed)->Self{self.seed=s;self}
    pub fn targeting(mut self,r:RealmId)->Self{self.target_realm=Some(r);self}
}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ExecutionOutcome{Success,Failure,Timeout}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ExecutionResult{pub agent_id:AgentId,pub session_id:SessionId,pub outcome:ExecutionOutcome,pub output:serde_json::Value,pub message:String,pub ticks_used:u64,pub completed_at:chrono::DateTime<chrono::Utc>}
impl ExecutionResult{
    pub fn success(a:AgentId,s:SessionId,o:serde_json::Value,t:u64)->Self{Self{agent_id:a,session_id:s,outcome:ExecutionOutcome::Success,output:o,message:"ok".into(),ticks_used:t,completed_at:chrono::Utc::now()}}
    pub fn failure(a:AgentId,s:SessionId,m:impl Into<String>)->Self{Self{agent_id:a,session_id:s,outcome:ExecutionOutcome::Failure,output:serde_json::Value::Null,message:m.into(),ticks_used:0,completed_at:chrono::Utc::now()}}
    pub fn is_success(&self)->bool{self.outcome==ExecutionOutcome::Success}
}
