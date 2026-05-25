use serde::{Deserialize, Serialize};
use bkg_core::{BkgResult, ContractId, EventId, ExecutionSeed, Hash256, LogicalTimestamp, RealmId, Signature};
use bkg_crypto::{hash::hash_event_fields, signing::PublicKey};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub realm: RealmId,
    pub timestamp: LogicalTimestamp,
    pub payload: serde_json::Value,
    pub execution_seed: ExecutionSeed,
    pub parent_hash: Hash256,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub contract_id: Option<ContractId>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub public_key: Option<PublicKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub signature: Option<Signature>,
    pub hash: Hash256,
}

impl Event {
    pub fn compute_hash(&self) -> Hash256 {
        let pb = serde_json::to_vec(&self.payload).unwrap_or_default();
        hash_event_fields(self.id.as_uuid().as_bytes(), self.realm.as_str(),
            self.timestamp.as_u64(), &pb, self.execution_seed.as_bytes(), self.parent_hash.as_bytes())
    }
    pub fn verify_hash(&self) -> bool { self.compute_hash() == self.hash }
    pub fn is_genesis(&self) -> bool { self.parent_hash.is_zero() }
    pub fn verify_signature(&self) -> BkgResult<bool> {
        match (&self.signature, &self.public_key) {
            (Some(sig), Some(pk)) => { bkg_crypto::verify_signature(self.hash.as_bytes(), sig, pk)?; Ok(true) }
            (None, _) => Ok(false),
            (Some(_), None) => Err(bkg_core::BkgError::SignatureError("sig without pk".into())),
        }
    }
}

pub struct EventBuilder {
    realm: RealmId, payload: serde_json::Value, seed: ExecutionSeed,
    parent: Hash256, timestamp: LogicalTimestamp, contract_id: Option<ContractId>,
}
impl EventBuilder {
    pub fn new(realm: RealmId) -> Self {
        Self { realm, payload: serde_json::Value::Null, seed: ExecutionSeed::random(),
               parent: Hash256::ZERO, timestamp: LogicalTimestamp::ZERO, contract_id: None }
    }
    pub fn payload(mut self, p: serde_json::Value) -> Self { self.payload = p; self }
    pub fn seed(mut self, s: ExecutionSeed) -> Self { self.seed = s; self }
    pub fn parent(mut self, h: Hash256) -> Self { self.parent = h; self }
    pub fn timestamp(mut self, t: LogicalTimestamp) -> Self { self.timestamp = t; self }
    pub fn contract(mut self, c: ContractId) -> Self { self.contract_id = Some(c); self }
    pub fn build(self) -> Event {
        let id = EventId::new();
        let pb = serde_json::to_vec(&self.payload).unwrap_or_default();
        let hash = hash_event_fields(id.as_uuid().as_bytes(), self.realm.as_str(),
            self.timestamp.as_u64(), &pb, self.seed.as_bytes(), self.parent.as_bytes());
        Event { id, realm: self.realm, timestamp: self.timestamp, payload: self.payload,
                execution_seed: self.seed, parent_hash: self.parent,
                contract_id: self.contract_id, public_key: None, signature: None, hash }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ev() -> Event { EventBuilder::new(RealmId::Styx).payload(serde_json::json!({"x":1})).build() }
    #[test] fn valid_hash() { assert!(ev().verify_hash()); }
    #[test] fn genesis() { assert!(ev().is_genesis()); }
    #[test] fn tamper_fails() { let mut e=ev(); e.payload=serde_json::json!({"x":2}); assert!(!e.verify_hash()); }
    #[test] fn serde() { let e=ev(); let j=serde_json::to_string(&e).unwrap(); let b:Event=serde_json::from_str(&j).unwrap(); assert_eq!(e.hash,b.hash); }
}
