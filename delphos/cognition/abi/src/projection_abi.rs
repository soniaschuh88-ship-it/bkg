// projection_abi.rs — read-model wire format.
// Projections served from bkg-projection over the API use this envelope.
use serde::{Deserialize, Serialize};
use crate::envelope::AbiEnvelope;

/// Wire format for a serialized projection read-model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionAbiPayload {
    pub projection_id: String,
    pub realm_id: String,
    pub state_version: u64,
    pub state_checksum: String,
    pub generated_at_lamport: u64,
    pub data: serde_json::Value,
}
pub type ProjectionEnvelope = AbiEnvelope<ProjectionAbiPayload>;

/// Subscription request: client asks for live projection updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionSubscription {
    pub subscriber_id: String,
    pub projection_ids: Vec<String>,
    pub from_version: u64,
}