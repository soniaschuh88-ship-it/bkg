use std::collections::HashMap;
use serde::{Deserialize,Serialize};
use bkg_core::{EventId,ExecutionSeed,Hash256,RealmId};
use bkg_event::Event;
use bkg_crypto::hash::hash_concatenated;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReconstructedState{pub events:Vec<Event>,pub cumulative_hash:Hash256,pub per_realm_state:HashMap<String,serde_json::Value>,pub seed:Option<ExecutionSeed>,pub terminal_event_id:Option<EventId>,pub replayed_at:chrono::DateTime<chrono::Utc>}
impl ReconstructedState{
    pub fn empty()->Self{Self{events:Vec::new(),cumulative_hash:Hash256::ZERO,per_realm_state:HashMap::new(),seed:None,terminal_event_id:None,replayed_at:chrono::Utc::now()}}
    pub fn advance(&mut self,e:&Event){self.cumulative_hash=hash_concatenated(&[self.cumulative_hash.as_bytes(),e.hash.as_bytes()]);self.per_realm_state.insert(e.realm.to_string(),e.payload.clone());if self.seed.is_none(){self.seed=Some(e.execution_seed);}self.terminal_event_id=Some(e.id);self.events.push(e.clone());}
    pub fn event_count(&self)->usize{self.events.len()}
    pub fn realm_state(&self,r:RealmId)->Option<&serde_json::Value>{self.per_realm_state.get(r.as_str())}
}
