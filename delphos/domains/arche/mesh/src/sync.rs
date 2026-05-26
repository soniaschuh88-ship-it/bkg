use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::node::MeshNodeId;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum SyncStatus{Pending,Applied,Rejected,Superseded}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SyncRecord{
    pub id:String, pub origin_node:MeshNodeId,
    pub entity_type:String, pub entity_id:String,
    pub realm_id:String, pub version:u64, pub lamport:u64, pub epoch:u64,
    pub data:serde_json::Value, pub checksum:String, pub status:SyncStatus,
    pub received_at:DateTime<Utc>,
}
impl SyncRecord{
    pub fn new(origin:MeshNodeId,entity_type:&str,entity_id:&str,realm:&str,version:u64,epoch:u64,data:serde_json::Value)->Self{
        let lamport=version;
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        entity_id.hash(&mut h); version.hash(&mut h); data.to_string().hash(&mut h);
        let checksum=format!("{:x}",std::hash::Hasher::finish(&h));
        Self{id:uuid::Uuid::new_v4().to_string(),origin_node:origin,entity_type:entity_type.into(),entity_id:entity_id.into(),realm_id:realm.into(),version,lamport,epoch,data,checksum,status:SyncStatus::Pending,received_at:Utc::now()}
    }
    pub fn apply(&mut self){self.status=SyncStatus::Applied;}
    pub fn reject(&mut self){self.status=SyncStatus::Rejected;}
}
