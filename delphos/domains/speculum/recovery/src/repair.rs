use serde::{Deserialize, Serialize};
use crate::crash::{CrashClassification, CrashReport};
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RepairStrategy { RollbackToSnapshot, ReplayFromLastGoodEvent, RequestManualIntervention, Skip }
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RepairOutcome { Repaired { strategy: RepairStrategy }, Failed { reason: String }, ManualRequired }
impl RepairOutcome { pub fn is_ok(&self) -> bool { matches!(self, Self::Repaired { .. }) } }
pub fn choose_strategy(r: &CrashReport) -> RepairStrategy {
    match r.classification {
        CrashClassification::HashChainBroken | CrashClassification::CapsuleCorrupted => RepairStrategy::RollbackToSnapshot,
        CrashClassification::PartialWrite | CrashClassification::MeshDesync => RepairStrategy::ReplayFromLastGoodEvent,
        _ => RepairStrategy::RequestManualIntervention,
    }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn broken_chain() { let r = CrashReport::new(CrashClassification::HashChainBroken, "t", ""); assert_eq!(choose_strategy(&r), RepairStrategy::RollbackToSnapshot); }
}
