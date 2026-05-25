use std::path::PathBuf;
use serde::{Deserialize,Serialize};
use bkg_core::{ContractId,EventId,ExecutionSeed,Hash256,RealmId,SessionId,Signature};
use crate::{ops::{ReadOp,WriteOp},trace::RealmTrace};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VerificationResult{pub hash_valid:bool,pub signature_valid:bool,#[serde(default,skip_serializing_if="Option::is_none")]pub detail:Option<String>}
impl VerificationResult{pub fn ok()->Self{Self{hash_valid:true,signature_valid:true,detail:None}}pub fn failed(d:impl Into<String>)->Self{Self{hash_valid:false,signature_valid:false,detail:Some(d.into())}}pub fn is_valid(&self)->bool{self.hash_valid&&self.signature_valid}}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Swd{
    pub session_id:SessionId,pub initiating_realm:RealmId,
    pub input_hash:Hash256,pub execution_seed:ExecutionSeed,
    pub realm_trace:Vec<RealmTrace>,pub contracts:Vec<ContractId>,
    pub touched_files:Vec<PathBuf>,pub write_ops:Vec<WriteOp>,pub read_ops:Vec<ReadOp>,
    pub provider_trace:Vec<String>,pub budget_used:u64,
    #[serde(default,skip_serializing_if="Option::is_none")]pub event_range_start:Option<EventId>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub event_range_end:Option<EventId>,
    pub hash_root:Hash256,
    #[serde(default,skip_serializing_if="Option::is_none")]pub signature:Option<Signature>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub verification_result:Option<VerificationResult>,
    pub started_at:chrono::DateTime<chrono::Utc>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub committed_at:Option<chrono::DateTime<chrono::Utc>>,
}
impl Swd{
    pub fn compute_hash_root(&self)->Hash256{let mut ops:Vec<Vec<u8>>=self.write_ops.iter().map(|o|serde_json::to_vec(o).unwrap_or_default()).collect();ops.extend(self.read_ops.iter().map(|o|serde_json::to_vec(o).unwrap_or_default()));bkg_crypto::hash::hash_swd_root(self.session_id.as_uuid().as_bytes(),&ops)}
    pub fn verify_hash_root(&self)->bool{self.compute_hash_root()==self.hash_root}
    pub fn op_count(&self)->usize{self.write_ops.len()+self.read_ops.len()}
    pub fn is_committed(&self)->bool{self.committed_at.is_some()}
}
