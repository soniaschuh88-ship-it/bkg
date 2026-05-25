use serde::{Deserialize,Serialize};
use bkg_core::{AgentId,CapsuleId,Capability,RealmId,SessionId};
use bkg_crypto::signing::PublicKey;
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AgentStatus{Idle,Running,Suspended,Terminated}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Agent{pub id:AgentId,pub realm:RealmId,pub name:String,pub capabilities:Vec<Capability>,pub capsule_id:CapsuleId,#[serde(default,skip_serializing_if="Option::is_none")]pub public_key:Option<PublicKey>,#[serde(default,skip_serializing_if="Option::is_none")]pub active_session:Option<SessionId>,pub status:AgentStatus,pub created_at:chrono::DateTime<chrono::Utc>,pub task_count:u64}
impl Agent{
    pub fn new(name:impl Into<String>,caps:Vec<Capability>,cid:CapsuleId,pk:Option<PublicKey>)->Self{Self{id:AgentId::new(),realm:RealmId::Telum,name:name.into(),capabilities:caps,capsule_id:cid,public_key:pk,active_session:None,status:AgentStatus::Idle,created_at:chrono::Utc::now(),task_count:0}}
    pub fn is_available(&self)->bool{self.status==AgentStatus::Idle}
    pub fn has_capability(&self,c:&Capability)->bool{self.capabilities.contains(c)}
}
