use bkg_capsule::Capsule;
use bkg_core::{BkgError,BkgResult,CapsuleId,Hash256};
use bkg_crypto::hash::hash_concatenated;
use crate::store::StateStore;
pub struct SledStore{db:sled::Db}
impl SledStore{
    pub fn open(p:impl AsRef<std::path::Path>)->BkgResult<Self>{Ok(Self{db:sled::open(p).map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?})}
    fn key(id:&CapsuleId,v:u64)->Vec<u8>{format!("{}/{:020}",id,v).into_bytes()}
    fn prefix(id:&CapsuleId)->Vec<u8>{format!("{}/",id).into_bytes()}
}
impl StateStore for SledStore{
    fn save(&self,c:&Capsule)->BkgResult<()>{self.db.insert(Self::key(&c.capsule_id,c.version),serde_json::to_vec(c)?).map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?;Ok(())}
    fn load_current(&self,id:&CapsuleId)->BkgResult<Option<Capsule>>{let last=self.db.scan_prefix(Self::prefix(id)).last().transpose().map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?;Ok(last.map(|(_,v)|serde_json::from_slice(&v)).transpose()?)}
    fn load_version(&self,id:&CapsuleId,v:u64)->BkgResult<Option<Capsule>>{Ok(self.db.get(Self::key(id,v)).map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?.map(|v|serde_json::from_slice(&v)).transpose()?)}
    fn load_history(&self,id:&CapsuleId)->BkgResult<Vec<Capsule>>{let mut out=Vec::new();for r in self.db.scan_prefix(Self::prefix(id)){let(_,v)=r.map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?;out.push(serde_json::from_slice(&v)?);}Ok(out)}
    fn capsule_count(&self)->BkgResult<usize>{let mut ids=std::collections::HashSet::new();for r in self.db.iter(){let(k,_)=r.map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?;if let Ok(s)=std::str::from_utf8(&k){if let Some(id)=s.split('/').next(){ids.insert(id.to_string());}}}Ok(ids.len())}
    fn snapshot_hash(&self)->BkgResult<Hash256>{
        let mut ids=std::collections::HashSet::new();
        for r in self.db.iter(){let(k,_)=r.map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?;if let Ok(s)=std::str::from_utf8(&k){if let Some(id)=s.split('/').next(){ids.insert(id.to_string());}}}
        let mut entries:Vec<(String,Hash256)>=Vec::new();
        for id_str in &ids{let prefix=format!("{id_str}/").into_bytes();if let Some(last)=self.db.scan_prefix(&prefix).last().transpose().map_err(|e|BkgError::Io(std::io::Error::other(e.to_string())))?{let c:Capsule=serde_json::from_slice(&last.1)?;entries.push((id_str.clone(),c.integrity_hash));}}
        entries.sort_by_key(|(id,_)|id.clone());
        let parts:Vec<Vec<u8>>=entries.iter().flat_map(|(id,h)|[id.as_bytes().to_vec(),h.0.to_vec()]).collect();
        let slices:Vec<&[u8]>=parts.iter().map(|p|p.as_slice()).collect();
        Ok(hash_concatenated(&slices))
    }
}
#[cfg(test)]mod tests{use super::*;use bkg_capsule::CapsuleManager;use bkg_core::RealmId;
    fn tmp()->std::path::PathBuf{let mut p=std::env::temp_dir();p.push(format!("bkg_sled_{}",uuid::Uuid::new_v4()));p}
    #[test]fn save_load(){let d=tmp();let s=SledStore::open(&d).unwrap();let mut m=CapsuleManager::new();let c=m.create(RealmId::Causa,None,serde_json::json!({"x":1})).unwrap();let id=c.capsule_id;s.save(&c).unwrap();assert!(s.load_current(&id).unwrap().unwrap().verify_integrity());let _=std::fs::remove_dir_all(&d);}
}
