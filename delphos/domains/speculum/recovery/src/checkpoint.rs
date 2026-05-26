use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RecoveryCheckpoint { pub id: String, pub realm_id: String, pub event_id: String, pub lamport: u64, pub state_checksum: String, pub created_at: DateTime<Utc> }
impl RecoveryCheckpoint { pub fn new(realm: impl Into<String>, event_id: impl Into<String>, lamport: u64, cksum: impl Into<String>) -> Self { Self { id: uuid::Uuid::new_v4().to_string(), realm_id: realm.into(), event_id: event_id.into(), lamport, state_checksum: cksum.into(), created_at: Utc::now() } } }
