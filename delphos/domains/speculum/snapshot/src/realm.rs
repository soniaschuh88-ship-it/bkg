use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
use bkg_core::RealmId;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RealmSnapshot{pub realm_id:RealmId,pub version:u64,pub entity_count:usize,pub checksum:String,pub data:serde_json::Value,pub captured_at:DateTime<Utc>}
impl RealmSnapshot{
    pub fn new(realm:RealmId,version:u64,data:serde_json::Value)->Self{
        use std::hash::Hash;
        let mut h=std::collections::hash_map::DefaultHasher::new();
        realm.as_str().hash(&mut h); version.hash(&mut h); data.to_string().hash(&mut h);
        let cksum=format!("{:x}",std::hash::Hasher::finish(&h));
        let count=data.as_object().map(|o|o.len()).unwrap_or(0);
        Self{realm_id:realm,version,entity_count:count,checksum:cksum,data,captured_at:Utc::now()}
    }
}
