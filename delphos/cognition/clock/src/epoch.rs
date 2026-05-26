// epoch.rs + timeline.rs — Epoch and Timeline types.
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;
use crate::tick::SequencedInstant;

/// Genesis tick + current tick for a realm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch { pub realm_id: RealmId, pub genesis: SequencedInstant, pub current: SequencedInstant }
impl Epoch {
    pub fn new(realm_id: RealmId) -> Self {
        let g = SequencedInstant::new(realm_id, 0, 0);
        Self { realm_id, genesis: g, current: g }
    }
    pub fn advance(&mut self, next: SequencedInstant) { self.current = next; }
    pub fn age(&self) -> u64 { self.current.lamport - self.genesis.lamport }
}
