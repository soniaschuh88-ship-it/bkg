use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use bkg_core::RealmId;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum DiffKind { Added, Removed, Modified { old: String, new: String } }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DiffEntry { pub entity_key: String, pub kind: DiffKind }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct StateDiff { pub realm_id: RealmId, pub from_version: u64, pub to_version: u64, pub entries: Vec<DiffEntry> }
impl StateDiff {
    pub fn compute(realm: RealmId, fv: u64, tv: u64, from: &BTreeMap<String, serde_json::Value>, to: &BTreeMap<String, serde_json::Value>) -> Self {
        let mut entries = Vec::new();
        for (k, nv) in to {
            match from.get(k) {
                None => entries.push(DiffEntry { entity_key: k.clone(), kind: DiffKind::Added }),
                Some(ov) if ov != nv => entries.push(DiffEntry { entity_key: k.clone(), kind: DiffKind::Modified { old: ov.to_string(), new: nv.to_string() } }),
                _ => {}
            }
        }
        for k in from.keys() { if !to.contains_key(k) { entries.push(DiffEntry { entity_key: k.clone(), kind: DiffKind::Removed }); } }
        Self { realm_id: realm, from_version: fv, to_version: tv, entries }
    }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn added_count(&self) -> usize { self.entries.iter().filter(|e| matches!(e.kind, DiffKind::Added)).count() }
    pub fn removed_count(&self) -> usize { self.entries.iter().filter(|e| matches!(e.kind, DiffKind::Removed)).count() }
}
#[cfg(test)]
mod tests { use super::*;
    fn m(p: &[(&str, &str)]) -> BTreeMap<String, serde_json::Value> { p.iter().map(|(k,v)|(k.to_string(), serde_json::json!(v))).collect() }
    #[test] fn added()   { assert_eq!(StateDiff::compute(RealmId::Telum,0,1,&m(&[]),&m(&[("T-1","t")])).added_count(),1); }
    #[test] fn removed() { assert_eq!(StateDiff::compute(RealmId::Telum,0,1,&m(&[("T-1","t")]),&m(&[])).removed_count(),1); }
    #[test] fn empty()   { assert!(StateDiff::compute(RealmId::Telum,0,0,&m(&[("x","y")]),&m(&[("x","y")])).is_empty()); }
}
