// causal.rs — VectorClock and CausalTime across multiple realms.
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;
use crate::tick::SequencedInstant;

/// Per-realm Lamport clock counters.
/// BTreeMap gives stable deterministic iteration order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VectorClock(BTreeMap<RealmId, u64>);

impl VectorClock {
    pub fn new() -> Self { Self::default() }

    /// Get current counter for a realm (0 if never seen).
    pub fn get(&self, realm: &RealmId) -> u64 { self.0.get(realm).copied().unwrap_or(0) }

    /// Advance the counter for a realm.
    pub fn tick(&mut self, realm: RealmId) -> u64 {
        let v = self.0.entry(realm).or_insert(0);
        *v += 1; *v
    }

    /// Merge two vector clocks: take the max per realm.
    pub fn merge(&mut self, other: &VectorClock) {
        for (realm, &cnt) in &other.0 {
            let entry = self.0.entry(*realm).or_insert(0);
            if cnt > *entry { *entry = cnt; }
        }
    }

    /// `self` happened-before `other`: every entry in self ≤ other, and at least one strictly less.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut at_least_one_less = false;
        for (realm, &cnt) in &self.0 {
            let other_cnt = other.get(realm);
            if cnt > other_cnt { return false; }
            if cnt < other_cnt { at_least_one_less = true; }
        }
        at_least_one_less
    }

    pub fn realm_count(&self) -> usize { self.0.len() }
}

/// Global causal time: maps realm → current SequencedInstant.
#[derive(Debug, Clone, Default)]
pub struct CausalTime { clocks: BTreeMap<RealmId, u64> }

impl CausalTime {
    pub fn new() -> Self { Self::default() }

    /// Advance the clock for `realm` and return the new instant.
    pub fn advance(&mut self, realm: RealmId) -> SequencedInstant {
        let lamport = {
            let v = self.clocks.entry(realm).or_insert(0);
            *v += 1; *v
        };
        // wall_nanos: use std time only for display
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        SequencedInstant::new(realm, lamport, wall)
    }

    pub fn current_lamport(&self, realm: &RealmId) -> u64 {
        self.clocks.get(realm).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn vector_tick() { let mut vc=VectorClock::new(); let r=RealmId::Telum; assert_eq!(vc.tick(r),1); assert_eq!(vc.tick(r),2); assert_eq!(vc.get(&r),2); }
    #[test] fn vector_merge() { let mut a=VectorClock::new(); let mut b=VectorClock::new(); let r=RealmId::Telum; a.tick(r); a.tick(r); b.tick(r); a.merge(&b); assert_eq!(a.get(&r),2); }
    #[test] fn happened_before() { let mut a=VectorClock::new(); let mut b=VectorClock::new(); let r=RealmId::Telum; a.tick(r); b.tick(r); b.tick(r); assert!(a.happened_before(&b)); assert!(!b.happened_before(&a)); }
    #[test] fn causal_time_advances() { let mut ct=CausalTime::new(); let r=RealmId::Telum; let t1=ct.advance(r); let t2=ct.advance(r); assert!(t1.happens_before(&t2)); }
}
