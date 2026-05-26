use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{policy::GcPolicy, pressure::GcPressure};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CompactionResult { pub events_compacted: u64, pub snapshots_sealed: u64, pub bytes_freed: u64, pub duration_ms: u64 }
impl CompactionResult { pub fn zero() -> Self { Self { events_compacted:0, snapshots_sealed:0, bytes_freed:0, duration_ms:0 } } }
pub struct GcRun { pub policy: GcPolicy, pub started_at: DateTime<Utc> }
impl GcRun {
    pub fn new(policy: GcPolicy) -> Self { Self { policy, started_at: Utc::now() } }
    pub fn run(&self, event_count: u64) -> CompactionResult {
        if !GcPressure::from_event_count(event_count).should_compact() || !self.policy.auto_compact { return CompactionResult::zero(); }
        let keep = self.policy.retention.min_events_to_keep;
        let compacted = event_count.saturating_sub(keep);
        CompactionResult { events_compacted: compacted, snapshots_sealed: 1, bytes_freed: compacted * 512, duration_ms: 0 }
    }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn no_compact() { assert_eq!(GcRun::new(GcPolicy::default_with_auto()).run(500).events_compacted, 0); }
    #[test] fn compact()    { assert!(GcRun::new(GcPolicy::default_with_auto()).run(1_000_000).events_compacted > 0); }
}
