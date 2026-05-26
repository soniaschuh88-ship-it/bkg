// capsule_abi.rs — capsule serialization contract.
// Defines the portable, versioned wire format for capsule interchange.
use serde::{Deserialize, Serialize};
use crate::envelope::AbiEnvelope;

/// Portable capsule export format — used for fork/export/restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleAbiPayload {
    pub capsule_id: String,
    pub realm_id: String,
    /// Entity type: task | mission | agent | session | etc.
    pub entity_type: String,
    pub entity_id: String,
    pub version: u64,
    pub state_checksum: String,
    pub events: Vec<serde_json::Value>,
    pub parent_capsule_id: Option<String>,
    pub sealed_at: Option<String>,
}
pub type CapsuleEnvelope = AbiEnvelope<CapsuleAbiPayload>;