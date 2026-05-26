use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use bkg_core::{BkgError,BkgResult};
use crate::{lane::LaneClass,packet::{BusPacket,PacketId},router::LaneRouter};

#[derive(Debug,Clone,thiserror::Error)]
pub enum BusError{#[error("lane full: {0}")] LaneFull(String)}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct BusStats{pub packets_sent:u64,pub packets_delivered:u64,pub packets_dropped:u64}

pub struct RealmBus{router:LaneRouter,stats:BusStats,realms:Vec<String>,seqs:HashMap<String,u64>}
impl Default for RealmBus{fn default()->Self{Self::new()}}
impl RealmBus{
    pub fn new()->Self{Self{router:LaneRouter::new(),stats:BusStats::default(),realms:Vec::new(),seqs:HashMap::new()}}
    pub fn register_realm(&mut self,r:impl Into<String>){self.realms.push(r.into());}
    pub fn send(&mut self,src:&str,tgt:&str,class:LaneClass,pt:&str,payload:serde_json::Value)->BkgResult<PacketId>{
        let k=format!("{src}→{tgt}");
        let seq=self.seqs.entry(k).or_insert(0);
        *seq+=1; let s=*seq;
        let p=BusPacket::new(src,tgt,class,s,s,pt,payload);
        let id=p.id.clone();
        self.router.send(p).map_err(|e|BkgError::Internal(format!("bus:{e}")))?;
        self.stats.packets_sent+=1;
        Ok(id)
    }
    pub fn recv(&mut self,target:&str)->Option<BusPacket>{
        let p=self.router.recv(target)?;
        self.stats.packets_delivered+=1;
        Some(p)
    }
    pub fn stats(&self)->&BusStats{&self.stats}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn roundtrip(){let mut b=RealmBus::new();b.send("telum","katoptron",LaneClass::Normal,"e",serde_json::json!({"id":"T-1"})).unwrap();let p=b.recv("katoptron").unwrap();assert_eq!(p.payload["id"],"T-1");assert_eq!(b.stats().packets_sent,1);}
    #[test] fn seq_increments(){let mut b=RealmBus::new();b.send("a","b",LaneClass::Normal,"e",serde_json::json!({})).unwrap();b.send("a","b",LaneClass::Normal,"e",serde_json::json!({})).unwrap();let p1=b.recv("b").unwrap();let p2=b.recv("b").unwrap();assert!(p2.sequence>p1.sequence);}
    #[test] fn empty(){let mut b=RealmBus::new();assert!(b.recv("x").is_none());}
}
