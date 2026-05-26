use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CapsuleDiff { pub capsule_id: String, pub from_version: u64, pub to_version: u64, pub checksum_changed: bool, pub entity_delta: i64 }
impl CapsuleDiff { pub fn new(id: &str, fv: u64, tv: u64, cksum: bool, delta: i64) -> Self { Self { capsule_id: id.into(), from_version: fv, to_version: tv, checksum_changed: cksum, entity_delta: delta } } }
