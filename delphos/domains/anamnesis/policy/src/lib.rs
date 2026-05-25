use serde::{Deserialize,Serialize};
use bkg_core::{BkgError,BkgResult};
use bkg_event::Event;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum PolicyVerdict{Allow,Warn(String),Deny(String)}
impl PolicyVerdict{pub fn is_deny(&self)->bool{matches!(self,PolicyVerdict::Deny(_))}}
pub trait Policy:Send+Sync{fn name(&self)->&str;fn evaluate(&self,e:&Event)->PolicyVerdict;}
#[derive(Default)]
pub struct PolicyEngine{policies:Vec<Box<dyn Policy>>}
impl PolicyEngine{
    pub fn new()->Self{Self::default()}
    pub fn add(&mut self,p:impl Policy+'static){self.policies.push(Box::new(p));}
    pub fn evaluate(&self,e:&Event)->BkgResult<Vec<PolicyVerdict>>{let mut vs=Vec::new();for p in &self.policies{let v=p.evaluate(e);if let PolicyVerdict::Deny(ref r)=v{return Err(BkgError::PolicyDenied(format!("[{}] {r}",p.name())));}vs.push(v);}Ok(vs)}
}
pub struct NoNullPayloadPolicy;
impl Policy for NoNullPayloadPolicy{fn name(&self)->&str{"no_null_payload"}fn evaluate(&self,e:&Event)->PolicyVerdict{if e.payload.is_null(){PolicyVerdict::Warn("null payload".into())}else{PolicyVerdict::Allow}}}
pub struct NonZeroSeedPolicy;
impl Policy for NonZeroSeedPolicy{fn name(&self)->&str{"non_zero_seed"}fn evaluate(&self,e:&Event)->PolicyVerdict{if e.execution_seed==bkg_core::ExecutionSeed::ZERO{PolicyVerdict::Warn("zero seed".into())}else{PolicyVerdict::Allow}}}
#[cfg(test)]mod tests{use super::*;use bkg_core::{ExecutionSeed,Hash256,RealmId};use bkg_event::EventBuilder;
    #[test]fn allow(){let mut e=PolicyEngine::new();e.add(NoNullPayloadPolicy);let ev=EventBuilder::new(RealmId::Telum).payload(serde_json::json!({"ok":true})).seed(ExecutionSeed::random()).parent(Hash256::ZERO).build();e.evaluate(&ev).unwrap();}
    #[test]fn null_warns(){let mut e=PolicyEngine::new();e.add(NoNullPayloadPolicy);let ev=EventBuilder::new(RealmId::Telum).seed(ExecutionSeed::random()).parent(Hash256::ZERO).build();let vs=e.evaluate(&ev).unwrap();assert!(vs.iter().any(|v|matches!(v,PolicyVerdict::Warn(_))));}
}
