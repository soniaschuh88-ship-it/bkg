use serde::{Deserialize, Serialize};
use crate::version::AbiVersion;

/// Stable symbol identifying the payload type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);
impl Symbol { pub fn new(s: impl Into<String>) -> Self { Self(s.into()) } }
impl std::fmt::Display for Symbol { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }

/// Every cross-system message in DELPHOS is wrapped in AbiEnvelope<T>.
/// This enables version negotiation for mesh + plugin + snapshot compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEnvelope<T> {
    /// ABI version of the sender.
    pub abi_version: AbiVersion,
    /// Stable type identifier for the payload.
    pub payload_type: Symbol,
    /// BLAKE3 hash of the serialized payload (integrity check).
    pub payload_hash: String,
    /// The actual payload.
    pub payload: T,
}

impl<T: Serialize> AbiEnvelope<T> {
    pub fn wrap(payload: T, payload_type: impl Into<String>) -> serde_json::Result<Self> {
        let json = serde_json::to_string(&payload)?;
        let hash = blake3::hash(json.as_bytes()).to_hex().to_string();
        Ok(Self { abi_version: AbiVersion::CURRENT, payload_type: Symbol::new(payload_type), payload_hash: hash, payload })
    }
    pub fn verify_hash(&self) -> bool {
        if let Ok(json) = serde_json::to_string(&self.payload) {
            let hash = blake3::hash(json.as_bytes()).to_hex().to_string();
            return hash == self.payload_hash;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn wrap_and_verify() {
        let env = AbiEnvelope::wrap(serde_json::json!({"msg":"hello BKG"}), "test.message").unwrap();
        assert_eq!(env.abi_version, AbiVersion::CURRENT);
        assert!(env.verify_hash());
    }
    #[test] fn symbol_display() { assert_eq!(Symbol::new("bkg.event").to_string(), "bkg.event"); }
}