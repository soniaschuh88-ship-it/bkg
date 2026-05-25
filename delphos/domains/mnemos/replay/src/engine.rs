use bkg_core::{BkgError,BkgResult,EventId};
use bkg_event::EventLedger;
use crate::state::ReconstructedState;
pub struct ReplayEngine;
impl ReplayEngine{
    pub fn reconstruct_state(ledger:&dyn EventLedger,up_to:Option<&EventId>)->BkgResult<ReconstructedState>{
        if let Some(id)=up_to{if ledger.get(id)?.is_none(){return Err(BkgError::EventNotFound(id.to_string()));}}
        let mut s=ReconstructedState::empty();
        for e in ledger.all_events(){if!e.verify_hash(){return Err(BkgError::CapsuleIntegrityError(format!("event {} bad hash",e.id)));}s.advance(e);if let Some(id)=up_to{if &e.id==id{break;}}}
        s.replayed_at=chrono::Utc::now();Ok(s)
    }
    pub fn reconstruct_until_ts(ledger:&dyn EventLedger,ts:u64)->BkgResult<ReconstructedState>{
        let mut s=ReconstructedState::empty();
        for e in ledger.all_events(){
            if e.timestamp.as_u64()>ts { break; }
            if !e.verify_hash(){return Err(BkgError::CapsuleIntegrityError(format!("event {} bad hash",e.id)));}
            s.advance(e);
        }
        Ok(s)
    }
}
#[cfg(test)]mod tests{use super::*;use bkg_core::{ExecutionSeed,Hash256,RealmId};use bkg_event::{EventBuilder,EventLedger,InMemoryLedger};
    fn build(n:usize)->InMemoryLedger{let mut l=InMemoryLedger::new();let s=ExecutionSeed::from_bytes([1u8;32]);let g=EventBuilder::new(RealmId::Styx).seed(s).payload(serde_json::json!({})).parent(Hash256::ZERO).build();let mut prev=g.clone();l.append(g).unwrap();for i in 1..n{let e=EventBuilder::new(RealmId::Telum).seed(s).payload(serde_json::json!({"i":i})).parent(prev.hash).timestamp(prev.timestamp.next()).build();prev=e.clone();l.append(e).unwrap();}l}
    #[test]fn full(){assert_eq!(ReplayEngine::reconstruct_state(&build(5),None).unwrap().event_count(),5);}
    #[test]fn partial(){let l=build(5);let id=l.all_events()[2].id;assert_eq!(ReplayEngine::reconstruct_state(&l,Some(&id)).unwrap().event_count(),3);}
    #[test]fn det(){let l=build(4);let h1=ReplayEngine::reconstruct_state(&l,None).unwrap().cumulative_hash;let h2=ReplayEngine::reconstruct_state(&l,None).unwrap().cumulative_hash;assert_eq!(h1,h2);}
    #[test]fn unknown_fails(){let l=build(3);assert!(ReplayEngine::reconstruct_state(&l,Some(&bkg_core::EventId::new())).is_err());}
    #[test]fn until_ts(){let l=build(5);assert_eq!(ReplayEngine::reconstruct_until_ts(&l,2).unwrap().event_count(),3);}
}
