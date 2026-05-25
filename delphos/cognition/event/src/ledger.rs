use bkg_core::{BkgResult, EventId, Hash256};
use crate::event::Event;
pub trait EventLedger: Send + Sync {
    fn append(&mut self, event: Event) -> BkgResult<()>;
    fn get(&self, id: &EventId) -> BkgResult<Option<&Event>>;
    fn head(&self) -> Option<&Event>;
    fn tail(&self) -> Option<&Event>;
    fn tail_hash(&self) -> Hash256 { self.tail().map(|e| e.hash).unwrap_or(Hash256::ZERO) }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn events_in_range(&self, from: u64, to: u64) -> Vec<&Event>;
    fn all_events(&self) -> Vec<&Event>;
}
