use serde::{Deserialize, Serialize};
use bkg_core::{BkgResult, Capability, ContractId, ExecutionSeed, Hash256, LogicalTimestamp, RealmId};
use bkg_crypto::hash::hash_concatenated;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ContractStatus { Pending, Validated, Delivered, Rejected, Expired }

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct CausalContract {
    pub id: ContractId, pub source_realm: RealmId, pub target_realm: RealmId,
    pub timestamp: LogicalTimestamp, pub payload: serde_json::Value,
    pub capabilities_required: Vec<Capability>, pub execution_seed: ExecutionSeed,
    #[serde(default,skip_serializing_if="Option::is_none")] pub ttl: Option<u64>,
    pub status: ContractStatus, pub hash: Hash256,
}

impl CausalContract {
    pub fn compute_hash(&self) -> Hash256 {
        let pb=serde_json::to_vec(&self.payload).unwrap_or_default();
        let cb=serde_json::to_vec(&self.capabilities_required).unwrap_or_default();
        hash_concatenated(&[self.id.as_uuid().as_bytes(),self.source_realm.as_str().as_bytes(),
            self.target_realm.as_str().as_bytes(),&self.timestamp.as_u64().to_le_bytes(),
            &pb,&cb,self.execution_seed.as_bytes()])
    }
    pub fn verify_hash(&self) -> bool { self.compute_hash() == self.hash }
    pub fn validate(&self) -> BkgResult<()> {
        if self.source_realm==self.target_realm { return Err(bkg_core::BkgError::InvalidContract{contract_id:self.id.to_string(),reason:"source==target".into()}); }
        if !self.verify_hash() { return Err(bkg_core::BkgError::InvalidContract{contract_id:self.id.to_string(),reason:"hash mismatch".into()}); }
        Ok(())
    }
    pub fn is_expired(&self, ts: LogicalTimestamp) -> bool {
        self.ttl.map(|t| ts.as_u64()>self.timestamp.as_u64().saturating_add(t)).unwrap_or(false)
    }
}

pub struct CausalContractBuilder {
    src: RealmId, tgt: RealmId, ts: LogicalTimestamp,
    payload: serde_json::Value, caps: Vec<Capability>, seed: ExecutionSeed, ttl: Option<u64>,
}
impl CausalContractBuilder {
    pub fn new(src: RealmId, tgt: RealmId) -> Self {
        Self{src,tgt,ts:LogicalTimestamp::ZERO,payload:serde_json::Value::Null,caps:Vec::new(),seed:ExecutionSeed::random(),ttl:None}
    }
    pub fn payload(mut self, p: serde_json::Value) -> Self { self.payload=p; self }
    pub fn timestamp(mut self, t: LogicalTimestamp) -> Self { self.ts=t; self }
    pub fn require(mut self, c: Capability) -> Self { self.caps.push(c); self }
    pub fn seed(mut self, s: ExecutionSeed) -> Self { self.seed=s; self }
    pub fn ttl(mut self, t: u64) -> Self { self.ttl=Some(t); self }
    pub fn build(self) -> CausalContract {
        let id=ContractId::new();
        let pb=serde_json::to_vec(&self.payload).unwrap_or_default();
        let cb=serde_json::to_vec(&self.caps).unwrap_or_default();
        let hash=hash_concatenated(&[id.as_uuid().as_bytes(),self.src.as_str().as_bytes(),
            self.tgt.as_str().as_bytes(),&self.ts.as_u64().to_le_bytes(),&pb,&cb,self.seed.as_bytes()]);
        CausalContract{id,source_realm:self.src,target_realm:self.tgt,timestamp:self.ts,
            payload:self.payload,capabilities_required:self.caps,execution_seed:self.seed,
            ttl:self.ttl,status:ContractStatus::Pending,hash}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(s:RealmId,t:RealmId)->CausalContract{CausalContractBuilder::new(s,t).payload(serde_json::json!({"op":"t"})).build()}
    #[test] fn valid(){c(RealmId::Telum,RealmId::Causa).validate().unwrap();}
    #[test] fn same_realm(){assert!(c(RealmId::Telum,RealmId::Telum).validate().is_err());}
    #[test] fn tamper(){let mut c=c(RealmId::Telum,RealmId::Causa);c.payload=serde_json::json!({});assert!(!c.verify_hash());}
    #[test] fn ttl(){let c=CausalContractBuilder::new(RealmId::Telum,RealmId::Mensa).ttl(5).timestamp(LogicalTimestamp(0)).build();assert!(!c.is_expired(LogicalTimestamp(5)));assert!(c.is_expired(LogicalTimestamp(6)));}
}
