// snapshot.rs — StateSnapshot: frozen RealmState for archival.
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::realm_state::RealmState;

/// Immutable frozen snapshot of a RealmState.
/// Used by bkg-gc for sealing and by bkg-recovery for restoration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub id: String,
    pub state: RealmState,
    pub checksum: String,
    pub sealed_at: DateTime<Utc>,
    pub gc_eligible: bool,
}

impl StateSnapshot {
    pub fn seal(state: RealmState) -> Self {
        let checksum = state.checksum();
        Self { id: uuid::Uuid::new_v4().to_string(), state, checksum, sealed_at: Utc::now(), gc_eligible: false }
    }
    pub fn mark_gc_eligible(&mut self) { self.gc_eligible = true; }
    pub fn verify(&self) -> bool { self.state.checksum() == self.checksum }
}

// mutation.rs — typed mutation record
// invariants.rs — compile-time + runtime invariant checks
// reconciliation.rs — repair utilities
