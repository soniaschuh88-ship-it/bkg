use serde::{Deserialize,Serialize};
use bkg_core::{AgentId,CapsuleId,EventId,Hash256,RealmId,SessionId};
use bkg_crypto::hash::hash_capsule;
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum CapsuleStatus{Active,Superseded,Retired}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Capsule{
    pub capsule_id:CapsuleId,pub realm:RealmId,
    #[serde(default,skip_serializing_if="Option::is_none")]pub agent_id:Option<AgentId>,
    pub state_snapshot:serde_json::Value,
    #[serde(default,skip_serializing_if="Option::is_none")]pub event_range_start:Option<EventId>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub event_range_end:Option<EventId>,
    pub swd_refs:Vec<SessionId>,pub memory_links:Vec<String>,
    pub version:u64,pub prev_hash:Hash256,pub integrity_hash:Hash256,
    pub status:CapsuleStatus,pub created_at:chrono::DateTime<chrono::Utc>,
}
impl Capsule{
    pub fn new(realm:RealmId,agent:Option<AgentId>,state:serde_json::Value)->Self{Self::build(CapsuleId::new(),realm,agent,state,1,Hash256::ZERO,vec![],vec![])}
    #[allow(clippy::too_many_arguments)]
    fn build(id:CapsuleId,realm:RealmId,agent:Option<AgentId>,state:serde_json::Value,version:u64,prev:Hash256,swd:Vec<SessionId>,mem:Vec<String>)->Self{
        let sj=serde_json::to_vec(&state).unwrap_or_default();
        let ih=hash_capsule(id.as_uuid().as_bytes(),version,&sj,&prev);
        Capsule{capsule_id:id,realm,agent_id:agent,state_snapshot:state,event_range_start:None,event_range_end:None,swd_refs:swd,memory_links:mem,version,prev_hash:prev,integrity_hash:ih,status:CapsuleStatus::Active,created_at:chrono::Utc::now()}
    }
    pub fn next_version(&self,state:serde_json::Value,swd:Option<SessionId>)->Capsule{let mut refs=self.swd_refs.clone();if let Some(s)=swd{refs.push(s);}Self::build(self.capsule_id,self.realm,self.agent_id,state,self.version+1,self.integrity_hash,refs,self.memory_links.clone())}
    pub fn compute_integrity_hash(&self)->Hash256{let sj=serde_json::to_vec(&self.state_snapshot).unwrap_or_default();hash_capsule(self.capsule_id.as_uuid().as_bytes(),self.version,&sj,&self.prev_hash)}
    pub fn verify_integrity(&self)->bool{self.compute_integrity_hash()==self.integrity_hash}
    pub fn is_first_version(&self)->bool{self.version==1}
}
#[cfg(test)]mod tests{use super::*;
    fn c()->Capsule{Capsule::new(RealmId::Causa,None,serde_json::json!({"v":0}))}
    #[test]fn valid(){assert!(c().verify_integrity());}
    #[test]fn chain(){let v1=c();let v2=v1.next_version(serde_json::json!({"v":1}),None);assert_eq!(v2.prev_hash,v1.integrity_hash);assert!(v2.verify_integrity());}
    #[test]fn tamper(){let mut c=c();c.state_snapshot=serde_json::json!({"t":1});assert!(!c.verify_integrity());}
}
