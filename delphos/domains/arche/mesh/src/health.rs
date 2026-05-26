use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::node::{MeshNodeId, NodeStatus};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NodeHealth{
    pub node_id:MeshNodeId, pub status:NodeStatus,
    pub lamport:u64, pub active_sessions:u32, pub active_leases:u32,
    pub entity_count:u64, pub last_heartbeat:DateTime<Utc>,
}
impl NodeHealth{
    pub fn new(node_id:MeshNodeId,lamport:u64)->Self{
        Self{node_id,status:NodeStatus::Unknown,lamport,active_sessions:0,active_leases:0,entity_count:0,last_heartbeat:Utc::now()}
    }
    pub fn heartbeat_age_secs(&self)->i64{(Utc::now()-self.last_heartbeat).num_seconds()}
    pub fn is_stale(&self,timeout_secs:i64)->bool{self.heartbeat_age_secs()>timeout_secs}
}
