use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
use crate::{limits::ResourceLimits, mount::VfsMount};
/// A point-in-time snapshot of VM state for rollback.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VmSnapshot{
    pub id:String, pub vm_id:String, pub label:String,
    pub limits:ResourceLimits, pub mounts:Vec<VfsMount>,
    pub env:std::collections::HashMap<String,String>,
    pub working_dir:String, pub checksum:String,
    pub created_at:DateTime<Utc>,
}
impl VmSnapshot{
    pub fn capture(vm_id:&str,label:&str,limits:ResourceLimits,mounts:Vec<VfsMount>,env:std::collections::HashMap<String,String>,working_dir:String)->Self{
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        vm_id.hash(&mut h); label.hash(&mut h); working_dir.hash(&mut h);
        let cksum=format!("{:x}",std::hash::Hasher::finish(&h));
        Self{id:uuid::Uuid::new_v4().to_string(),vm_id:vm_id.into(),label:label.into(),limits,mounts,env,working_dir,checksum:cksum,created_at:Utc::now()}
    }
}
