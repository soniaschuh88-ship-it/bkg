use bkg_core::Hash256;
use bkg_event::EventLedger;
use crate::report::{CheckResult,VerificationReport};
pub struct ChainVerificationResult{pub report:VerificationReport,pub events_verified:usize,pub first_broken_index:Option<usize>}
impl ChainVerificationResult{pub fn is_valid(&self)->bool{self.report.is_passed()}}
pub fn verify_hash_chain(ledger:&dyn EventLedger)->ChainVerificationResult{
    let mut r=VerificationReport::new("styx_ledger");
    let all=ledger.all_events();
    if all.is_empty(){r.record(CheckResult::warn("non_empty","empty"));return ChainVerificationResult{report:r,events_verified:0,first_broken_index:None};}
    let mut broken=None;let mut exp=Hash256::ZERO;
    for(i,e)in all.iter().enumerate(){
        if!e.verify_hash(){r.record(CheckResult::fail("self_hash",format!("event {} idx {i}",e.id)));if broken.is_none(){broken=Some(i);}}
        if e.parent_hash!=exp{r.record(CheckResult::fail("parent_hash",format!("event {} idx {i} parent mismatch",e.id)));if broken.is_none(){broken=Some(i);}}
        exp=e.hash;
    }
    if broken.is_none(){r.record(CheckResult::pass(format!("chain_{}_events",all.len())));}
    ChainVerificationResult{report:r,events_verified:all.len(),first_broken_index:broken}
}
#[cfg(test)]mod tests{use super::*;use bkg_core::{ExecutionSeed,Hash256,RealmId};use bkg_event::{EventBuilder,EventLedger,InMemoryLedger};
    fn build(n:usize)->InMemoryLedger{let mut l=InMemoryLedger::new();let s=ExecutionSeed::from_bytes([1u8;32]);let g=EventBuilder::new(RealmId::Styx).seed(s).payload(serde_json::json!({})).parent(Hash256::ZERO).build();let mut p=g.clone();l.append(g).unwrap();for i in 1..n{let e=EventBuilder::new(RealmId::Telum).seed(s).payload(serde_json::json!({"i":i})).parent(p.hash).timestamp(p.timestamp.next()).build();p=e.clone();l.append(e).unwrap();}l}
    #[test]fn valid(){assert!(verify_hash_chain(&build(5)).is_valid());}
    #[test]fn empty_warns(){assert!(!verify_hash_chain(&InMemoryLedger::new()).is_valid());}
}
