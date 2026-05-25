use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Capability {
    LedgerRead,LedgerWrite,CapsuleWrite,CapsuleRead,CapsuleReadForeign,
    AgentControl,RuntimeExecute,PolicyEnforce,PolicyOverride,
    MemoryRead,MemoryWrite,ReplayTrigger,ContractIssue,ContractRoute,Observe,Verify,
}
impl Capability {
    pub fn all()->&'static [Capability]{&[Capability::LedgerRead,Capability::LedgerWrite,Capability::CapsuleWrite,Capability::CapsuleRead,Capability::CapsuleReadForeign,Capability::AgentControl,Capability::RuntimeExecute,Capability::PolicyEnforce,Capability::PolicyOverride,Capability::MemoryRead,Capability::MemoryWrite,Capability::ReplayTrigger,Capability::ContractIssue,Capability::ContractRoute,Capability::Observe,Capability::Verify]}
    pub fn is_privileged(self)->bool{matches!(self,Capability::PolicyOverride|Capability::CapsuleReadForeign|Capability::ContractRoute)}
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize,Default)]
pub struct CapabilitySet(Vec<Capability>);
impl CapabilitySet {
    pub fn new(caps:impl IntoIterator<Item=Capability>)->Self{let mut v:Vec<_>=caps.into_iter().collect();v.sort_by_key(|c|format!("{c:?}"));v.dedup();Self(v)}
    pub fn contains(&self,c:Capability)->bool{self.0.contains(&c)}
    pub fn satisfies(&self,req:&[Capability])->bool{req.iter().all(|r|self.contains(*r))}
    pub fn as_slice(&self)->&[Capability]{&self.0}
    pub fn all()->Self{Self::new(Capability::all().iter().copied())}
}
