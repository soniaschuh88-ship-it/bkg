use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
use crate::reality::SnapshotId;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TimelineSnapshot{pub id:SnapshotId,pub label:String,pub fork_depth:u32,pub origin_snapshot_id:Option<SnapshotId>,pub event_range:(u64,u64),pub created_at:DateTime<Utc>}
impl TimelineSnapshot{
    pub fn new(label:impl Into<String>,origin:Option<SnapshotId>,from_event:u64,to_event:u64)->Self{Self{id:SnapshotId::new(),label:label.into(),fork_depth:origin.is_some() as u32,origin_snapshot_id:origin,event_range:(from_event,to_event),created_at:Utc::now()}}
    pub fn event_count(&self)->u64{self.event_range.1.saturating_sub(self.event_range.0)}
}
