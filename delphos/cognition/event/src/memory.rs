use std::collections::HashMap;
use bkg_core::{BkgError, BkgResult, EventId};
use crate::{event::Event, ledger::EventLedger};

pub struct InMemoryLedger { events: Vec<Event>, index: HashMap<EventId, usize> }
impl InMemoryLedger {
    pub fn new() -> Self { Self { events: Vec::new(), index: HashMap::new() } }
    pub fn from_events(events: Vec<Event>) -> BkgResult<Self> {
        let mut l = Self::new(); for e in events { l.append(e)?; } Ok(l)
    }
}
impl Default for InMemoryLedger { fn default() -> Self { Self::new() } }
impl EventLedger for InMemoryLedger {
    fn append(&mut self, event: Event) -> BkgResult<()> {
        if !event.verify_hash() { return Err(BkgError::CapsuleIntegrityError(format!("event {} bad hash",event.id))); }
        if self.index.contains_key(&event.id) { return Err(BkgError::DuplicateEventId(event.id.to_string())); }
        let exp = self.tail_hash();
        if event.parent_hash != exp { return Err(BkgError::HashChainBroken{event_id:event.id.to_string(),expected:exp.to_hex(),actual:event.parent_hash.to_hex()}); }
        let idx = self.events.len();
        self.index.insert(event.id, idx);
        self.events.push(event);
        Ok(())
    }
    fn get(&self, id: &EventId) -> BkgResult<Option<&Event>> { Ok(self.index.get(id).map(|&i|&self.events[i])) }
    fn head(&self) -> Option<&Event> { self.events.first() }
    fn tail(&self) -> Option<&Event> { self.events.last() }
    fn len(&self) -> usize { self.events.len() }
    fn events_in_range(&self, from: u64, to: u64) -> Vec<&Event> {
        self.events.iter().filter(|e|{let t=e.timestamp.as_u64();t>=from&&t<=to}).collect()
    }
    fn all_events(&self) -> Vec<&Event> { self.events.iter().collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::{ExecutionSeed, Hash256, RealmId};
    use crate::event::EventBuilder;
    fn g() -> Event { EventBuilder::new(RealmId::Styx).seed(ExecutionSeed::from_bytes([0;32])).payload(serde_json::json!({})).parent(Hash256::ZERO).build() }
    fn n(p:&Event)->Event { EventBuilder::new(RealmId::Telum).seed(ExecutionSeed::random()).payload(serde_json::json!({})).parent(p.hash).timestamp(p.timestamp.next()).build() }
    #[test] fn append_get() { let mut l=InMemoryLedger::new();let e=g();let id=e.id;l.append(e).unwrap();assert!(l.get(&id).unwrap().is_some()); }
    #[test] fn chain() { let mut l=InMemoryLedger::new();let e0=g();let e1=n(&e0);l.append(e0).unwrap();l.append(e1).unwrap();assert_eq!(l.len(),2); }
    #[test] fn bad_parent() { let mut l=InMemoryLedger::new();let e0=g();l.append(e0).unwrap();let bad=EventBuilder::new(RealmId::Telum).seed(ExecutionSeed::random()).payload(serde_json::json!({})).parent(Hash256([0xFF;32])).timestamp(bkg_core::LogicalTimestamp(1)).build();assert!(l.append(bad).is_err()); }
    #[test] fn dup() { let mut l=InMemoryLedger::new();let e=g();let c=e.clone();l.append(e).unwrap();assert!(l.append(c).is_err()); }
}
