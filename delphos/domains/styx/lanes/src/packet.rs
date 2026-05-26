use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::lane::LaneClass;

#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct PacketId(pub String);
impl PacketId{pub fn new()->Self{Self(uuid::Uuid::new_v4().to_string())}}
impl Default for PacketId{fn default()->Self{Self::new()}}
impl std::fmt::Display for PacketId{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.0)}}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum PacketStatus{Queued,InFlight,Delivered,Failed,Dropped}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BusPacket {
    pub id:PacketId, pub source_realm:String, pub target_realm:String,
    pub lane_class:LaneClass, pub sequence:u64, pub sender_lamport:u64,
    pub payload_type:String, pub payload:serde_json::Value, pub payload_hash:String,
    pub signature:Option<String>, pub status:PacketStatus,
    pub created_at:DateTime<Utc>, pub delivered_at:Option<DateTime<Utc>>,
    pub causal_parent:Option<PacketId>,
}
impl BusPacket {
    pub fn new(src:impl Into<String>,tgt:impl Into<String>,class:LaneClass,seq:u64,lamport:u64,pt:impl Into<String>,payload:serde_json::Value)->Self {
        let pt=pt.into();
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        seq.hash(&mut h); pt.hash(&mut h); payload.to_string().hash(&mut h);
        let hash=format!("{:x}",std::hash::Hasher::finish(&h));
        Self{id:PacketId::new(),source_realm:src.into(),target_realm:tgt.into(),lane_class:class,sequence:seq,sender_lamport:lamport,payload_type:pt,payload,payload_hash:hash,signature:None,status:PacketStatus::Queued,created_at:Utc::now(),delivered_at:None,causal_parent:None}
    }
    pub fn verify_hash(&self)->bool{
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        self.sequence.hash(&mut h);self.payload_type.hash(&mut h);self.payload.to_string().hash(&mut h);
        format!("{:x}",std::hash::Hasher::finish(&h))==self.payload_hash
    }
    pub fn mark_delivered(&mut self){self.status=PacketStatus::Delivered;self.delivered_at=Some(Utc::now());}
    pub fn with_parent(mut self,p:PacketId)->Self{self.causal_parent=Some(p);self}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create_verify(){let p=BusPacket::new("a","b",LaneClass::Normal,1,1,"e",serde_json::json!({}));assert!(p.verify_hash());assert_eq!(p.status,PacketStatus::Queued);}
    #[test] fn delivered(){let mut p=BusPacket::new("a","b",LaneClass::High,1,1,"e",serde_json::json!({}));p.mark_delivered();assert_eq!(p.status,PacketStatus::Delivered);}
    #[test] fn causal(){let p1=BusPacket::new("a","b",LaneClass::Normal,1,1,"e",serde_json::json!({}));let p2=BusPacket::new("a","b",LaneClass::Normal,2,2,"e",serde_json::json!({})).with_parent(p1.id.clone());assert_eq!(p2.causal_parent.unwrap().0,p1.id.0);}
}
