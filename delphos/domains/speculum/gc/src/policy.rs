use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RetentionPolicy { pub keep_snapshots: u32, pub max_event_age_days: u32, pub min_events_to_keep: u64 }
impl Default for RetentionPolicy { fn default() -> Self { Self { keep_snapshots: 10, max_event_age_days: 90, min_events_to_keep: 1000 } } }
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct GcPolicy { pub retention: RetentionPolicy, pub auto_compact: bool, pub compact_threshold_mb: u64 }
impl GcPolicy { pub fn default_with_auto() -> Self { Self { retention: RetentionPolicy::default(), auto_compact: true, compact_threshold_mb: 512 } } }
