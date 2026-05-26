use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use bkg_core::{BkgError,BkgResult};
use crate::node::MeshNodeId;

#[derive(Debug,Clone,thiserror::Error)]
pub enum LeaseError{
    #[error("lease held by {holder}, epoch {epoch}")] Contested{holder:String,epoch:u64},
    #[error("lease expired")] Expired,
    #[error("stale epoch: current {current}, got {incoming}")] StaleEpoch{current:u64,incoming:u64},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MeshLease{
    pub resource_id:String, pub holder_id:MeshNodeId,
    pub epoch:u64, pub granted_at:DateTime<Utc>, pub expires_at:DateTime<Utc>,
}
impl MeshLease{
    pub fn new(resource:impl Into<String>,holder:MeshNodeId,epoch:u64,ttl_secs:i64)->Self{
        let now=Utc::now();
        Self{resource_id:resource.into(),holder_id:holder,epoch,granted_at:now,expires_at:now+Duration::seconds(ttl_secs)}
    }
    pub fn is_expired(&self)->bool{Utc::now()>self.expires_at}
    pub fn is_held_by(&self,node:&MeshNodeId)->bool{&self.holder_id==node&&!self.is_expired()}
    pub fn renew(&mut self,ttl_secs:i64){self.expires_at=Utc::now()+Duration::seconds(ttl_secs);}
}

#[derive(Debug,Default)]
pub struct LeaseRegistry{leases:std::collections::HashMap<String,MeshLease>}
impl LeaseRegistry{
    pub fn new()->Self{Self::default()}
    pub fn acquire(&mut self,resource:&str,node:MeshNodeId,epoch:u64,ttl_secs:i64)->BkgResult<MeshLease>{
        if matches!(self.leases.get(resource), Some(e) if !e.is_expired() && epoch <= e.epoch) { return Err(BkgError::Internal("lease contested".into())); }
        let lease=MeshLease::new(resource,node,epoch,ttl_secs);
        self.leases.insert(resource.to_string(),lease.clone());
        Ok(lease)
    }
    pub fn release(&mut self,resource:&str,node:&MeshNodeId)->bool{
        if matches!(self.leases.get(resource), Some(l) if l.holder_id==*node) { self.leases.remove(resource); true } else { false }
    }
    pub fn get(&self,resource:&str)->Option<&MeshLease>{self.leases.get(resource)}
    pub fn recover_abandoned(&mut self)->usize{
        let expired:Vec<String>=self.leases.iter().filter(|(_,l)|l.is_expired()).map(|(k,_)|k.clone()).collect();
        let n=expired.len(); for k in expired{self.leases.remove(&k);} n
    }
}

#[cfg(test)]
mod tests{use super::*;
    #[test] fn acquire_and_release(){
        let mut r=LeaseRegistry::new();
        let n=MeshNodeId::new();
        let l=r.acquire("task-T1",n.clone(),1,60).unwrap();
        assert!(!l.is_expired());
        assert!(r.release("task-T1",&n));
    }
    #[test] fn higher_epoch_wins(){
        let mut r=LeaseRegistry::new();
        let n1=MeshNodeId::new(); let n2=MeshNodeId::new();
        r.acquire("res",n1,1,60).unwrap();
        assert!(r.acquire("res",n2,2,60).is_ok());
    }
    #[test] fn same_epoch_contested(){
        let mut r=LeaseRegistry::new();
        let n1=MeshNodeId::new(); let n2=MeshNodeId::new();
        r.acquire("res",n1,1,60).unwrap();
        assert!(r.acquire("res",n2,1,60).is_err());
    }
    #[test] fn expired_recoverable(){
        let mut r=LeaseRegistry::new();
        // Insert an already-expired lease directly
        let l=MeshLease::new("res",MeshNodeId::new(),1,-1);
        r.leases.insert("res".to_string(),l);
        assert_eq!(r.recover_abandoned(),1);
    }
}
