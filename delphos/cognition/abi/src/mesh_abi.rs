// mesh_abi.rs — cross-node replication format.
// All mesh sync operations use this ABI for version-safe interchange.
use serde::{Deserialize, Serialize};
use crate::envelope::AbiEnvelope;

/// A replicated state item (any entity type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSyncPayload {
    pub origin_node_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub realm_id: String,
    pub version: u64,
    pub lamport: u64,
    pub data: serde_json::Value,
    pub checksum: String,
    /// Epoch for lease fencing.
    pub epoch: u64,
}

/// Node health ping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshHeartbeat {
    pub node_id: String,
    pub lamport: u64,
    pub crate_count: u32,
    pub active_sessions: u32,
}

pub type MeshSyncEnvelope = AbiEnvelope<MeshSyncPayload>;
pub type MeshHeartbeatEnvelope = AbiEnvelope<MeshHeartbeat>;