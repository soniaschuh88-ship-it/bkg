// packet_abi.rs — IPC packet format for bkg-lanes Realm Bus.
// Every inter-realm message is a signed, sequenced, replayable packet.
use serde::{Deserialize, Serialize};
use crate::envelope::AbiEnvelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanePriority { Background, Low, Normal, High, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusPacket {
    pub packet_id: String,
    pub source_realm: String,
    pub target_realm: String,
    pub priority: LanePriority,
    pub sequence: u64,
    pub payload_type: String,
    pub payload: serde_json::Value,
    /// Optional HMAC-SHA256 signature for authenticity.
    pub signature: Option<String>,
}
impl BusPacket {
    pub fn new(src: &str, tgt: &str, priority: LanePriority, seq: u64, payload_type: &str, payload: serde_json::Value) -> Self {
        Self { packet_id: uuid::Uuid::new_v4().to_string(), source_realm: src.into(), target_realm: tgt.into(), priority, sequence: seq, payload_type: payload_type.into(), payload, signature: None }
    }
    pub fn is_signed(&self) -> bool { self.signature.is_some() }
}
pub type PacketEnvelope = AbiEnvelope<BusPacket>;
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create_packet() {
        let p = BusPacket::new("telum","katoptron",LanePriority::Normal,1,"task.event",serde_json::json!({}));
        assert!(!p.is_signed()); assert_eq!(p.sequence,1);
    }
}