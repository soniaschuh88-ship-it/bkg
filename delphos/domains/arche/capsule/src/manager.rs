use std::collections::HashMap;
use bkg_core::{AgentId,BkgError,BkgResult,CapsuleId,RealmId,SessionId};
use crate::capsule::{Capsule,CapsuleStatus};
pub struct CapsuleManager{history:HashMap<CapsuleId,Vec<Capsule>>}
impl CapsuleManager{
    pub fn new()->Self{Self{history:HashMap::new()}}
    pub fn create(&mut self,realm:RealmId,agent:Option<AgentId>,state:serde_json::Value)->BkgResult<Capsule>{let c=Capsule::new(realm,agent,state);self.history.entry(c.capsule_id).or_default().push(c.clone());Ok(c)}
    pub fn update(&mut self,id:CapsuleId,state:serde_json::Value,swd:Option<SessionId>)->BkgResult<Capsule>{
        let h=self.history.get_mut(&id).ok_or_else(||BkgError::CapsuleNotFound(id.to_string()))?;
        let cur=h.last().ok_or_else(||BkgError::CapsuleNotFound(id.to_string()))?;
        if cur.status==CapsuleStatus::Retired{return Err(BkgError::CapsuleIntegrityError("retired".into()));}
        let next=cur.next_version(state,swd);
        if let Some(prev)=h.last_mut(){prev.status=CapsuleStatus::Superseded;}
        h.push(next.clone());Ok(next)
    }
    pub fn retire(&mut self,id:CapsuleId)->BkgResult<()>{let h=self.history.get_mut(&id).ok_or_else(||BkgError::CapsuleNotFound(id.to_string()))?;h.last_mut().ok_or_else(||BkgError::CapsuleNotFound(id.to_string()))?.status=CapsuleStatus::Retired;Ok(())}
    pub fn current(&self,id:&CapsuleId)->BkgResult<Option<&Capsule>>{Ok(self.history.get(id).and_then(|h|h.last()))}
    pub fn history(&self,id:&CapsuleId)->Vec<&Capsule>{self.history.get(id).map(|h|h.iter().collect()).unwrap_or_default()}
    pub fn len(&self)->usize{self.history.len()}
    pub fn is_empty(&self)->bool{self.history.is_empty()}
}
impl Default for CapsuleManager{fn default()->Self{Self::new()}}
#[cfg(test)]mod tests{use super::*;use bkg_core::RealmId;
    fn create(m:&mut CapsuleManager)->CapsuleId{m.create(RealmId::Causa,None,serde_json::json!({"v":0})).unwrap().capsule_id}
    #[test]fn create_read(){let mut m=CapsuleManager::new();let id=create(&mut m);assert_eq!(m.current(&id).unwrap().unwrap().version,1);}
    #[test]fn update_inc(){let mut m=CapsuleManager::new();let id=create(&mut m);m.update(id,serde_json::json!({}),None).unwrap();assert_eq!(m.current(&id).unwrap().unwrap().version,2);}
    #[test]fn retired_blocks(){let mut m=CapsuleManager::new();let id=create(&mut m);m.retire(id).unwrap();assert!(m.update(id,serde_json::json!({}),None).is_err());}
    #[test]fn history(){let mut m=CapsuleManager::new();let id=create(&mut m);m.update(id,serde_json::json!({}),None).unwrap();assert_eq!(m.history(&id).len(),2);}
}
