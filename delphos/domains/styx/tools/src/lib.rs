use bkg_core::RealmId;
use bkg_event::EventLedger;
use serde_json::{json,Value};
pub fn ledger_summary(l:&dyn EventLedger)->Value{let all=l.all_events();let mut counts=std::collections::HashMap::new();for e in &all{*counts.entry(e.realm.to_string()).or_insert(0)+=1;}json!({"total_events":l.len(),"tail_hash":l.tail().map(|e|e.hash.to_hex()),"events_per_realm":counts})}
pub fn dump_realm(l:&dyn EventLedger,r:RealmId)->Value{let events:Vec<Value>=l.all_events().iter().filter(|e|e.realm==r).map(|e|json!({"id":e.id.to_string(),"ts":e.timestamp.as_u64(),"payload":e.payload})).collect();json!({"realm":r.to_string(),"events":events})}
#[cfg(test)]mod tests{use super::*;use bkg_core::{ExecutionSeed,Hash256,RealmId};use bkg_event::{EventBuilder,EventLedger,InMemoryLedger};
    fn build()->InMemoryLedger{let mut l=InMemoryLedger::new();let e=EventBuilder::new(RealmId::Styx).seed(ExecutionSeed::random()).payload(serde_json::json!({})).parent(Hash256::ZERO).build();l.append(e).unwrap();l}
    #[test]fn summary(){assert_eq!(ledger_summary(&build())["total_events"],1);}
    #[test]fn dump(){let l=build();assert_eq!(dump_realm(&l,RealmId::Styx)["events"].as_array().unwrap().len(),1);}
}
