// clock.rs — RealmClock: per-realm deterministic tick source.
use bkg_core::RealmId;
use crate::{causal::CausalTime, tick::SequencedInstant};

#[derive(Debug, thiserror::Error)]
pub enum ClockError {
    #[error("duplicate lamport counter {0} in realm — determinism failure")]
    DuplicateLamport(u64),
    #[error("clock went backwards: was {was}, got {got}")]
    ClockReversed { was: u64, got: u64 },
}

/// Per-realm deterministic clock.
pub struct RealmClock { realm_id: RealmId, causal: CausalTime, last: u64 }

impl RealmClock {
    pub fn new(realm_id: RealmId) -> Self {
        Self { realm_id, causal: CausalTime::new(), last: 0 }
    }

    /// Advance clock and return new `SequencedInstant`.
    pub fn tick(&mut self) -> Result<SequencedInstant, ClockError> {
        let instant = self.causal.advance(self.realm_id);
        if instant.lamport == self.last {
            return Err(ClockError::DuplicateLamport(instant.lamport));
        }
        if instant.lamport < self.last {
            return Err(ClockError::ClockReversed { was: self.last, got: instant.lamport });
        }
        self.last = instant.lamport;
        Ok(instant)
    }

    pub fn realm_id(&self) -> RealmId { self.realm_id }
    pub fn current_lamport(&self) -> u64 { self.last }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn monotone() { let mut c=RealmClock::new(RealmId::Telum); let t1=c.tick().unwrap(); let t2=c.tick().unwrap(); assert!(t1.happens_before(&t2)); assert_eq!(c.current_lamport(),2); }
    #[test] fn realm_id_consistent() { let r=RealmId::Telum; let mut c=RealmClock::new(r); let t=c.tick().unwrap(); assert_eq!(t.realm_id,r); }
}
