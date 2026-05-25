use serde::{Deserialize,Serialize};
use bkg_core::{EventId,Hash256};
use crate::state::ReconstructedState;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BranchReport{pub diverged:bool,pub divergence_index:Option<usize>,pub divergence_event_a:Option<EventId>,pub divergence_event_b:Option<EventId>,pub hash_a:Hash256,pub hash_b:Hash256,pub event_count_a:usize,pub event_count_b:usize,pub description:String}
pub struct DivergenceDetector;
impl DivergenceDetector{
    pub fn detect(a:&ReconstructedState,b:&ReconstructedState)->Option<BranchReport>{
        if a.cumulative_hash==b.cumulative_hash&&a.event_count()==b.event_count(){return None;}
        let min=a.events.len().min(b.events.len());
        let idx=(0..min).find(|&i|a.events[i].hash!=b.events[i].hash);
        let(ea,eb,ha,hb)=match idx{Some(i)=>(Some(a.events[i].id),Some(b.events[i].id),a.events[i].hash,b.events[i].hash),None=>(a.events.get(min).map(|e|e.id),b.events.get(min).map(|e|e.id),a.cumulative_hash,b.cumulative_hash)};
        let desc=match idx{Some(i)=>format!("hashes differ at {i}: {ha} vs {hb}"),None=>format!("counts differ: {} vs {}",a.event_count(),b.event_count())};
        Some(BranchReport{diverged:true,divergence_index:idx,divergence_event_a:ea,divergence_event_b:eb,hash_a:ha,hash_b:hb,event_count_a:a.event_count(),event_count_b:b.event_count(),description:desc})
    }
}
#[cfg(test)]mod tests{use super::*;use bkg_core::{ExecutionSeed,Hash256,RealmId};use bkg_event::{EventBuilder,EventLedger,InMemoryLedger};use crate::engine::ReplayEngine;
    fn build(n:usize)->InMemoryLedger{let mut l=InMemoryLedger::new();let s=ExecutionSeed::from_bytes([1u8;32]);let g=EventBuilder::new(RealmId::Styx).seed(s).payload(serde_json::json!({})).parent(Hash256::ZERO).build();let mut p=g.clone();l.append(g).unwrap();for i in 1..n{let e=EventBuilder::new(RealmId::Telum).seed(s).payload(serde_json::json!({"i":i})).parent(p.hash).timestamp(p.timestamp.next()).build();p=e.clone();l.append(e).unwrap();}l}
    #[test]fn no_div(){let l=build(4);let s1=ReplayEngine::reconstruct_state(&l,None).unwrap();let s2=ReplayEngine::reconstruct_state(&l,None).unwrap();assert!(DivergenceDetector::detect(&s1,&s2).is_none());}
    #[test]fn div(){let l=build(5);let full=ReplayEngine::reconstruct_state(&l,None).unwrap();let tid=l.all_events()[2].id;let part=ReplayEngine::reconstruct_state(&l,Some(&tid)).unwrap();assert!(DivergenceDetector::detect(&full,&part).unwrap().diverged);}
}
