use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum CrashClassification { HashChainBroken, CapsuleCorrupted, ReplayDiverged, PartialWrite, MeshDesync, UnknownFailure }
impl CrashClassification { pub fn is_recoverable(self) -> bool { !matches!(self, Self::UnknownFailure | Self::ReplayDiverged) } }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CrashReport { pub id: String, pub classification: CrashClassification, pub realm_id: String, pub last_good_event: Option<String>, pub detected_at: DateTime<Utc>, pub details: String }
impl CrashReport {
    pub fn new(c: CrashClassification, realm: impl Into<String>, details: impl Into<String>) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), classification: c, realm_id: realm.into(), last_good_event: None, detected_at: Utc::now(), details: details.into() }
    }
    pub fn is_recoverable(&self) -> bool { self.classification.is_recoverable() }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn recoverable()     { assert!(CrashClassification::HashChainBroken.is_recoverable()); }
    #[test] fn not_recoverable() { assert!(!CrashClassification::ReplayDiverged.is_recoverable()); }
}
