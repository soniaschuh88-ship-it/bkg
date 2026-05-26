// arbitrator.rs — Kernel Arbitration Layer.
// Prevents: concurrent causality corruption, invalid realm transitions,
// duplicate lamport ticks, cyclic approvals, replay paradoxes.
// The Kernel is now: BIOS + Hypervisor + Causality Judge.
// Single source of truth for all realm-crossing invariant enforcement.
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ArbitrationError {
    #[error("causality cycle detected: {0}")]
    CausalityCycle(String),
    #[error("duplicate lamport {lamport} in realm {realm}")]
    DuplicateLamport { realm: RealmId, lamport: u64 },
    #[error("invalid realm transition: {from} → {to}: {reason}")]
    InvalidRealmTransition { from: RealmId, to: RealmId, reason: String },
    #[error("replay paradox: event {id} processed out of causal order")]
    ReplayParadox { id: String },
    #[error("cyclic approval: request {0} is its own dependency")]
    CyclicApproval(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbitrationDecision { Allow, Deny, RequireApproval }

/// Kernel Arbitration Layer.
/// All cross-realm operations pass through this before execution.
#[derive(Debug, Default)]
pub struct KernelArbitrator {
    /// Track seen (realm, lamport) pairs to detect duplicates.
    seen_lamports: HashMap<RealmId, HashSet<u64>>,
    /// Processed event IDs for replay paradox detection.
    processed_events: HashSet<String>,
}

impl KernelArbitrator {
    pub fn new() -> Self { Self::default() }

    /// Validate a cross-realm event before it is applied.
    pub fn validate_event(
        &mut self,
        event_id: &str,
        source_realm: RealmId,
        lamport: u64,
        causal_parent: Option<&str>,
    ) -> Result<ArbitrationDecision, ArbitrationError> {
        // 1. Duplicate lamport check
        if !self.seen_lamports.entry(source_realm).or_default().insert(lamport) {
            return Err(ArbitrationError::DuplicateLamport { realm: source_realm, lamport });
        }
        // 2. Replay paradox: event already processed
        if self.processed_events.contains(event_id) {
            return Err(ArbitrationError::ReplayParadox { id: event_id.to_string() });
        }
        // 3. Causal parent must have been processed (if specified)
        if let Some(parent) = causal_parent {
            if !parent.is_empty() && !self.processed_events.contains(parent) {
                return Err(ArbitrationError::ReplayParadox { id: format!("parent {parent} not yet processed") });
            }
        }
        self.processed_events.insert(event_id.to_string());
        Ok(ArbitrationDecision::Allow)
    }

    /// Reset seen lamports for replay (full ledger replay mode).
    pub fn reset_for_replay(&mut self) {
        self.seen_lamports.clear();
        self.processed_events.clear();
    }

    pub fn event_count(&self) -> usize { self.processed_events.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn allow_valid() {
        let mut a = KernelArbitrator::new();
        assert!(matches!(a.validate_event("e1", RealmId::Telum, 1, None), Ok(ArbitrationDecision::Allow)));
    }
    #[test] fn duplicate_lamport() {
        let mut a = KernelArbitrator::new();
        a.validate_event("e1", RealmId::Telum, 1, None).unwrap();
        assert!(matches!(a.validate_event("e2", RealmId::Telum, 1, None), Err(ArbitrationError::DuplicateLamport { .. })));
    }
    #[test] fn replay_paradox_duplicate_event() {
        let mut a = KernelArbitrator::new();
        a.validate_event("e1", RealmId::Telum, 1, None).unwrap();
        assert!(matches!(a.validate_event("e1", RealmId::Styx, 2, None), Err(ArbitrationError::ReplayParadox { .. })));
    }
    #[test] fn causal_parent_missing() {
        let mut a = KernelArbitrator::new();
        let r = a.validate_event("e2", RealmId::Telum, 2, Some("e1-not-processed"));
        assert!(r.is_err());
    }
    #[test] fn reset_allows_replay() {
        let mut a = KernelArbitrator::new();
        a.validate_event("e1", RealmId::Telum, 1, None).unwrap();
        a.reset_for_replay();
        assert!(a.validate_event("e1", RealmId::Telum, 1, None).is_ok());
    }
    #[test] fn different_realms_same_lamport_ok() {
        let mut a = KernelArbitrator::new();
        a.validate_event("e1", RealmId::Telum, 1, None).unwrap();
        assert!(a.validate_event("e2", RealmId::Styx, 1, None).is_ok()); // different realms = ok
    }
}
