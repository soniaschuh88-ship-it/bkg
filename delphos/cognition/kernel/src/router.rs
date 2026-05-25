use std::collections::{HashMap, HashSet};
use bkg_core::{BkgError, BkgResult, LogicalTimestamp, RealmId};
use bkg_contracts::{CausalContract, ContractStatus};

#[derive(Debug, Clone)]
pub struct RoutePolicy { denied: HashSet<(RealmId,RealmId)>, allow_all: bool }
impl RoutePolicy {
    pub fn allow_all() -> Self { Self { denied: HashSet::new(), allow_all: true } }
    pub fn deny(mut self, s: RealmId, t: RealmId) -> Self { self.denied.insert((s,t)); self }
    pub fn is_permitted(&self, s: RealmId, t: RealmId) -> bool { !self.denied.contains(&(s,t)) && self.allow_all }
}
impl Default for RoutePolicy { fn default() -> Self { Self::allow_all() } }

pub struct RealmRouter { policy: RoutePolicy, pending: HashMap<String,CausalContract>, delivered: HashSet<String> }
impl RealmRouter {
    pub fn new(p: RoutePolicy) -> Self { Self{policy:p,pending:HashMap::new(),delivered:HashSet::new()} }
    pub fn default_open() -> Self { Self::new(RoutePolicy::allow_all()) }

    pub fn dispatch(&mut self, mut c: CausalContract) -> BkgResult<()> {
        c.validate()?;
        if !self.policy.is_permitted(c.source_realm,c.target_realm) {
            return Err(BkgError::RealmBoundaryViolation{from_realm:c.source_realm,to_realm:c.target_realm});
        }
        c.status=ContractStatus::Validated;
        self.pending.insert(c.id.to_string(),c);
        Ok(())
    }
    pub fn pending_for(&self, r: RealmId) -> Vec<&CausalContract> { self.pending.values().filter(|c|c.target_realm==r).collect() }
    pub fn acknowledge(&mut self, id: &str) -> BkgResult<()> {
        let mut c=self.pending.remove(id).ok_or_else(||BkgError::InvalidContract{contract_id:id.into(),reason:"not pending".into()})?;
        c.status=ContractStatus::Delivered; self.delivered.insert(id.to_string()); Ok(())
    }
    pub fn expire_stale(&mut self, ts: LogicalTimestamp) -> Vec<String> {
        let ids:Vec<String>=self.pending.values().filter(|c|c.is_expired(ts)).map(|c|c.id.to_string()).collect();
        for id in &ids { self.pending.remove(id); }
        ids
    }
    pub fn pending_count(&self) -> usize { self.pending.len() }
}
impl Default for RealmRouter { fn default() -> Self { Self::default_open() } }

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_contracts::CausalContractBuilder;
    use bkg_core::{Capability, RealmId};
    fn c(s:RealmId,t:RealmId)->CausalContract{CausalContractBuilder::new(s,t).require(Capability::CapsuleRead).build()}
    #[test] fn dispatch_ack() { let mut r=RealmRouter::default_open();let ct=c(RealmId::Telum,RealmId::Causa);let id=ct.id.to_string();r.dispatch(ct).unwrap();r.acknowledge(&id).unwrap();assert_eq!(r.pending_count(),0); }
    #[test] fn denied() { let mut r=RealmRouter::new(RoutePolicy::allow_all().deny(RealmId::Katoptron,RealmId::Telum));assert!(r.dispatch(c(RealmId::Katoptron,RealmId::Telum)).is_err()); }
    #[test] fn ttl() { let mut r=RealmRouter::default_open();let ct=CausalContractBuilder::new(RealmId::Telum,RealmId::Mensa).ttl(5).timestamp(LogicalTimestamp(0)).build();r.dispatch(ct).unwrap();assert_eq!(r.expire_stale(LogicalTimestamp(10)).len(),1); }
}
