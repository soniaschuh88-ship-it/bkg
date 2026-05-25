use std::{collections::HashMap,sync::{Arc,RwLock}};
use bkg_capsule::Capsule;
use bkg_core::{BkgResult,CapsuleId,Hash256};
use bkg_crypto::hash::hash_concatenated;
use crate::store::StateStore;
#[derive(Default,Clone)]
pub struct InMemoryStore{inner:Arc<RwLock<HashMap<String,Vec<Capsule>>>>}
impl InMemoryStore{pub fn new()->Self{Self::default()}}
impl StateStore for InMemoryStore{
    fn save(&self,c:&Capsule)->BkgResult<()>{let mut m=self.inner.write().unwrap();let v=m.entry(c.capsule_id.to_string()).or_default();if!v.iter().any(|x|x.version==c.version){v.push(c.clone());v.sort_by_key(|x|x.version);}Ok(())}
    fn load_current(&self,id:&CapsuleId)->BkgResult<Option<Capsule>>{Ok(self.inner.read().unwrap().get(&id.to_string()).and_then(|v|v.last().cloned()))}
    fn load_version(&self,id:&CapsuleId,v:u64)->BkgResult<Option<Capsule>>{Ok(self.inner.read().unwrap().get(&id.to_string()).and_then(|vs|vs.iter().find(|c|c.version==v).cloned()))}
    fn load_history(&self,id:&CapsuleId)->BkgResult<Vec<Capsule>>{Ok(self.inner.read().unwrap().get(&id.to_string()).cloned().unwrap_or_default())}
    fn capsule_count(&self)->BkgResult<usize>{Ok(self.inner.read().unwrap().len())}
    fn snapshot_hash(&self)->BkgResult<Hash256>{
        let m=self.inner.read().unwrap();
        let mut entries:Vec<_>=m.iter().filter_map(|(id,vs)|vs.last().map(|c|(id.clone(),c.integrity_hash))).collect();
        entries.sort_by_key(|(id,_)|id.clone());
        let parts:Vec<Vec<u8>>=entries.iter().flat_map(|(id,h)|[id.as_bytes().to_vec(),h.0.to_vec()]).collect();
        let slices:Vec<&[u8]>=parts.iter().map(|p|p.as_slice()).collect();
        Ok(hash_concatenated(&slices))
    }
}
#[cfg(test)]mod tests{use super::*;use bkg_capsule::CapsuleManager;use bkg_core::RealmId;
    fn create(s:&InMemoryStore)->CapsuleId{let mut m=CapsuleManager::new();let c=m.create(RealmId::Causa,None,serde_json::json!({"x":1})).unwrap();let id=c.capsule_id;s.save(&c).unwrap();id}
    #[test]fn save_load(){let s=InMemoryStore::new();let id=create(&s);assert!(s.load_current(&id).unwrap().unwrap().verify_integrity());}
    #[test]fn idempotent(){let s=InMemoryStore::new();let id=create(&s);let c=s.load_current(&id).unwrap().unwrap();s.save(&c).unwrap();assert_eq!(s.load_history(&id).unwrap().len(),1);}
    #[test]fn snapshot_changes(){
        let s=InMemoryStore::new();let mut m=CapsuleManager::new();
        let c1=m.create(RealmId::Causa,None,serde_json::json!({"v":1})).unwrap();let id=c1.capsule_id;s.save(&c1).unwrap();
        let h1=s.snapshot_hash().unwrap();
        let c2=m.update(id,serde_json::json!({"v":2}),None).unwrap();s.save(&c2).unwrap();
        assert_ne!(h1,s.snapshot_hash().unwrap());
    }
}
