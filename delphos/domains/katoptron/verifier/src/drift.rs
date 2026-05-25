use bkg_replay::{DivergenceDetector,ReconstructedState};
use crate::report::{CheckResult,VerificationReport};
pub fn detect_drift(a:&ReconstructedState,b:&ReconstructedState)->VerificationReport{
    let mut r=VerificationReport::new("replay_drift");
    match DivergenceDetector::detect(a,b){None=>r.record(CheckResult::pass("replay_equivalence")),Some(br)=>r.record(CheckResult::fail("replay_divergence",br.description.clone()))}
    r
}
#[cfg(test)]mod tests{use super::*;use bkg_core::{ExecutionSeed,Hash256,RealmId};use bkg_event::{EventBuilder,EventLedger,InMemoryLedger};use bkg_replay::ReplayEngine;
    fn build(n:usize)->InMemoryLedger{let mut l=InMemoryLedger::new();let s=ExecutionSeed::from_bytes([1u8;32]);let g=EventBuilder::new(RealmId::Styx).seed(s).payload(serde_json::json!({})).parent(Hash256::ZERO).build();let mut p=g.clone();l.append(g).unwrap();for i in 1..n{let e=EventBuilder::new(RealmId::Telum).seed(s).payload(serde_json::json!({"i":i})).parent(p.hash).timestamp(p.timestamp.next()).build();p=e.clone();l.append(e).unwrap();}l}
    #[test]fn no_drift(){let l=build(4);let s1=ReplayEngine::reconstruct_state(&l,None).unwrap();let s2=ReplayEngine::reconstruct_state(&l,None).unwrap();assert!(detect_drift(&s1,&s2).is_passed());}
    #[test]fn drift(){let l=build(5);let full=ReplayEngine::reconstruct_state(&l,None).unwrap();let tid=l.all_events()[2].id;let part=ReplayEngine::reconstruct_state(&l,Some(&tid)).unwrap();assert!(!detect_drift(&full,&part).is_passed());}
}
