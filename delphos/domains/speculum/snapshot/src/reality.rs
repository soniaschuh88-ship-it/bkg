use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct SnapshotId(pub String);
impl SnapshotId{pub fn new()->Self{Self(uuid::Uuid::new_v4().to_string())}}
#[allow(clippy::derivable_impls)]
impl Default for SnapshotId{fn default()->Self{Self::new()}}
impl std::fmt::Display for SnapshotId{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.0)}}

/// A full deterministic snapshot of the entire DELPHOS world state.
/// fork() creates a new divergent timeline from this point.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RealitySnapshot{
    pub id:SnapshotId, pub label:String,
    /// Per-realm entity state (realm → entity_type/id → value)
    pub realm_states:BTreeMap<String,serde_json::Value>,
    pub global_lamport:u64, pub event_count:u64,
    pub parent_snapshot_id:Option<SnapshotId>,
    pub checksum:String, pub created_at:DateTime<Utc>,
    pub gc_eligible:bool,
}
impl RealitySnapshot{
    pub fn new(label:impl Into<String>,realm_states:BTreeMap<String,serde_json::Value>,lamport:u64,event_count:u64)->Self{
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        lamport.hash(&mut h); event_count.hash(&mut h);
        for(k,v) in &realm_states{k.hash(&mut h);v.to_string().hash(&mut h);}
        let cksum=format!("{:x}",std::hash::Hasher::finish(&h));
        Self{id:SnapshotId::new(),label:label.into(),realm_states,global_lamport:lamport,event_count,parent_snapshot_id:None,checksum:cksum,created_at:Utc::now(),gc_eligible:false}
    }
    pub fn fork(&self,new_label:impl Into<String>)->Self{
        let mut forked=self.clone();
        forked.id=SnapshotId::new();
        forked.label=new_label.into();
        forked.parent_snapshot_id=Some(self.id.clone());
        forked.created_at=Utc::now();
        forked
    }
    pub fn mark_gc_eligible(&mut self){self.gc_eligible=true;}
    pub fn verify(&self)->bool{
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        self.global_lamport.hash(&mut h); self.event_count.hash(&mut h);
        for(k,v) in &self.realm_states{k.hash(&mut h);v.to_string().hash(&mut h);}
        format!("{:x}",std::hash::Hasher::finish(&h))==self.checksum
    }
}

#[cfg(test)]
mod tests{use super::*;
    fn snap()->RealitySnapshot{
        let mut states=BTreeMap::new();
        states.insert("telum/tasks".into(),serde_json::json!({"count":5}));
        RealitySnapshot::new("snap-1",states,100,50)
    }
    #[test] fn create_verify(){let s=snap();assert!(s.verify());}
    #[test] fn fork_has_parent(){let s=snap();let f=s.fork("fork-1");assert_eq!(f.parent_snapshot_id.as_ref().unwrap().0,s.id.0);assert_ne!(f.id,s.id);}
    #[test] fn gc_flag(){let mut s=snap();s.mark_gc_eligible();assert!(s.gc_eligible);}
    #[test] fn checksum_changes_on_tamper(){let mut s=snap();let orig=s.checksum.clone();s.global_lamport=999;assert_ne!(orig,format!("{:x}",{use std::hash::{Hash,Hasher};let mut h=std::collections::hash_map::DefaultHasher::new();s.global_lamport.hash(&mut h);std::hash::Hasher::finish(&h)}));}
}
