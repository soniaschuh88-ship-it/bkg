// tick.rs — SequencedInstant: deterministic time value.
// wall_nanos is DISPLAY ONLY — never used for ordering.
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

/// The only legal time value in DELPHOS business logic.
///
/// Ordering is determined entirely by `(realm_id, lamport)`.
/// `wall_nanos` exists solely for human-readable display — never compare it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequencedInstant {
    pub realm_id: RealmId,
    pub lamport: u64,
    /// Display-only. Do NOT use for ordering or replay decisions.
    pub wall_nanos: u64,
}

impl SequencedInstant {
    pub fn new(realm_id: RealmId, lamport: u64, wall_nanos: u64) -> Self {
        Self { realm_id, lamport, wall_nanos }
    }

    /// Happens-before: `self` causally precedes `other` in the same realm.
    pub fn happens_before(&self, other: &Self) -> bool {
        self.realm_id == other.realm_id && self.lamport < other.lamport
    }

    /// Concurrent: same realm, equal lamport = determinism failure.
    pub fn is_concurrent_with(&self, other: &Self) -> bool {
        self.realm_id == other.realm_id && self.lamport == other.lamport
    }
}

impl PartialOrd for SequencedInstant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.realm_id != other.realm_id { return None; }
        Some(self.lamport.cmp(&other.lamport))
    }
}

impl std::fmt::Display for SequencedInstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.realm_id, self.lamport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn realm() -> RealmId { RealmId::Telum }
    #[test] fn ordering() { let r=realm(); let a=SequencedInstant::new(r,1,0); let b=SequencedInstant::new(r,2,0); assert!(a.happens_before(&b)); assert!(!b.happens_before(&a)); }
    #[test] fn concurrent_same_lamport() { let r=realm(); let a=SequencedInstant::new(r,5,0); let b=SequencedInstant::new(r,5,0); assert!(a.is_concurrent_with(&b)); }
    #[test] fn cross_realm_no_order() { let a=SequencedInstant::new(RealmId::Telum,1,0); let b=SequencedInstant::new(RealmId::Styx,1,0); assert!(a.partial_cmp(&b).is_none()); }
    #[test] fn display() { let r=realm(); let t=SequencedInstant::new(r,42,0); assert!(t.to_string().contains("42")); }
}
