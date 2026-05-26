use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct MeshNodeId(pub String);
impl MeshNodeId{pub fn new()->Self{Self(uuid::Uuid::new_v4().to_string())}}
impl Default for MeshNodeId{fn default()->Self{Self::new()}}
impl std::fmt::Display for MeshNodeId{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.0)}}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]
#[serde(rename_all="snake_case")]
pub enum NodeStatus{#[default]Unknown,Online,Offline,Degraded,Recovering}
impl NodeStatus{pub fn is_healthy(self)->bool{matches!(self,Self::Online|Self::Degraded)}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MeshNode{
    pub id:MeshNodeId, pub label:String, pub address:String,
    pub status:NodeStatus, pub capabilities:Vec<String>,
    pub joined_at:DateTime<Utc>, pub last_seen:DateTime<Utc>,
    pub lamport:u64, pub epoch:u64,
}
impl MeshNode{
    pub fn new(label:impl Into<String>,address:impl Into<String>)->Self{
        let now=Utc::now();
        Self{id:MeshNodeId::new(),label:label.into(),address:address.into(),
             status:NodeStatus::Unknown,capabilities:vec![],
             joined_at:now,last_seen:now,lamport:0,epoch:0}
    }
    pub fn mark_online(&mut self){self.status=NodeStatus::Online;self.last_seen=Utc::now();}
    pub fn mark_offline(&mut self){self.status=NodeStatus::Offline;}
    pub fn tick(&mut self)->u64{self.lamport+=1;self.lamport}
    pub fn seconds_since_seen(&self)->i64{(Utc::now()-self.last_seen).num_seconds()}
}

#[cfg(test)]
mod tests{use super::*;
    #[test] fn create_online(){let mut n=MeshNode::new("node-0","127.0.0.1:9000");n.mark_online();assert_eq!(n.status,NodeStatus::Online);}
    #[test] fn tick_increments(){let mut n=MeshNode::new("n","a");assert_eq!(n.tick(),1);assert_eq!(n.tick(),2);}
    #[test] fn offline(){let mut n=MeshNode::new("n","a");n.mark_online();n.mark_offline();assert!(!n.status.is_healthy());}
}
