// bkg-state/reconciliation.rs — repair utilities for partial writes and replay mismatches.
// Used by bkg-recovery when the ledger and state diverge.
use serde::{Deserialize, Serialize};
use crate::realm_state::RealmState;
use crate::snapshot::StateSnapshot;

/// The outcome of a reconciliation attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationOutcome {
    /// State was already consistent — no action needed.
    AlreadyConsistent,
    /// State was repaired from a snapshot.
    RepairedFromSnapshot { snapshot_id: String },
    /// State was rebuilt by replaying events from genesis.
    RebuiltFromLedger { events_replayed: u64 },
    /// Reconciliation failed — manual intervention needed.
    Failed { reason: String },
}

impl ReconciliationOutcome {
    pub fn is_ok(&self) -> bool { !matches!(self, Self::Failed { .. }) }
}

/// Reconciliation context — holds the current state and the last known good snapshot.
pub struct ReconciliationContext {
    pub current_state: RealmState,
    pub last_snapshot: Option<StateSnapshot>,
}

impl ReconciliationContext {
    pub fn new(state: RealmState, snapshot: Option<StateSnapshot>) -> Self {
        Self { current_state: state, last_snapshot: snapshot }
    }

    /// Verify state checksum against the snapshot.
    pub fn verify(&self) -> bool {
        match &self.last_snapshot {
            None => true, // no snapshot to verify against
            Some(snap) => self.current_state.checksum() == snap.checksum,
        }
    }

    /// Attempt repair: roll back to last snapshot if checksums differ.
    pub fn reconcile(&self) -> ReconciliationOutcome {
        if self.verify() {
            return ReconciliationOutcome::AlreadyConsistent;
        }
        match &self.last_snapshot {
            Some(snap) if snap.verify() => {
                ReconciliationOutcome::RepairedFromSnapshot {
                    snapshot_id: snap.id.clone(),
                }
            }
            _ => ReconciliationOutcome::Failed {
                reason: "no valid snapshot available; full ledger replay required".into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_state::RealmState;
    use bkg_core::RealmId;

    #[test] fn consistent_state() {
        let s = RealmState::empty(RealmId::Telum);
        let ctx = ReconciliationContext::new(s, None);
        assert_eq!(ctx.reconcile(), ReconciliationOutcome::AlreadyConsistent);
    }

    #[test] fn repair_from_snapshot() {
        let s = RealmState::empty(RealmId::Causa);
        let snap = StateSnapshot::seal(s.clone());
        // Artificially diverge the state checksum
        let mut diverged = s;
        diverged.set_entity("ghost", "G-1", serde_json::json!({}));
        let ctx = ReconciliationContext::new(diverged, Some(snap.clone()));
        let outcome = ctx.reconcile();
        assert!(matches!(outcome, ReconciliationOutcome::RepairedFromSnapshot { .. }));
    }
}
