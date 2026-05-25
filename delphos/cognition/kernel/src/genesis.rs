use serde::{Deserialize, Serialize};
use bkg_core::{BkgError, BkgResult, ExecutionSeed, Hash256, LogicalTimestamp, RealmId};
use bkg_event::{Event, EventBuilder, EventLedger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genesis {
    pub event: Event,
    pub locked_hash: Hash256,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Genesis {
    pub fn initialise(seed: ExecutionSeed, ledger: &mut dyn EventLedger) -> BkgResult<Self> {
        if !ledger.is_empty() { return Err(BkgError::Internal("ledger not empty".into())); }
        let event = EventBuilder::new(RealmId::Styx)
            .payload(serde_json::json!({"type":"genesis","version":"0.1.0","seed":seed.to_hex()}))
            .seed(seed).parent(Hash256::ZERO).timestamp(LogicalTimestamp::ZERO).build();
        let locked_hash = event.hash;
        ledger.append(event.clone())?;
        Ok(Genesis { event, locked_hash, created_at: chrono::Utc::now() })
    }

    pub fn restore(event: Event, locked_hash: Hash256) -> BkgResult<Self> {
        if event.hash != locked_hash || !event.verify_hash() { return Err(BkgError::GenesisMutationAttempt); }
        Ok(Genesis { event, locked_hash, created_at: chrono::Utc::now() })
    }

    pub fn verify(&self) -> BkgResult<()> {
        if !self.event.verify_hash() || self.event.hash != self.locked_hash {
            return Err(BkgError::GenesisMutationAttempt);
        }
        Ok(())
    }

    pub fn hash(&self) -> &Hash256 { &self.locked_hash }
    pub fn timestamp(&self) -> LogicalTimestamp { self.event.timestamp }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_event::InMemoryLedger;
    #[test] fn init() { let mut l=InMemoryLedger::new(); let g=Genesis::initialise(ExecutionSeed::random(),&mut l).unwrap(); assert_eq!(l.len(),1); g.verify().unwrap(); }
    #[test] fn double_init_fails() { let s=ExecutionSeed::random(); let mut l=InMemoryLedger::new(); Genesis::initialise(s,&mut l).unwrap(); assert!(Genesis::initialise(s,&mut l).is_err()); }
    #[test] fn tamper() { let mut l=InMemoryLedger::new(); let mut g=Genesis::initialise(ExecutionSeed::random(),&mut l).unwrap(); g.event.payload=serde_json::json!({"t":1}); assert!(g.verify().is_err()); }
    #[test] fn restore_ok() { let mut l=InMemoryLedger::new(); let g=Genesis::initialise(ExecutionSeed::random(),&mut l).unwrap(); Genesis::restore(g.event.clone(),g.locked_hash).unwrap(); }
}
