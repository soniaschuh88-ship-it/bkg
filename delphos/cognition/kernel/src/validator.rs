use bkg_core::{BkgError, BkgResult, Capability, LogicalTimestamp};
use bkg_contracts::CausalContract;

pub struct CausalContractValidator;
impl CausalContractValidator {
    pub fn check(&self, c: &CausalContract, caps: &[Capability], ts: LogicalTimestamp) -> BkgResult<()> {
        if !c.verify_hash() { return Err(BkgError::InvalidContract{contract_id:c.id.to_string(),reason:"hash mismatch".into()}); }
        if c.source_realm==c.target_realm { return Err(BkgError::InvalidContract{contract_id:c.id.to_string(),reason:"src==tgt".into()}); }
        for req in &c.capabilities_required { if !caps.contains(req) { return Err(BkgError::MissingCapability(format!("{req:?}"))); } }
        if c.is_expired(ts) { return Err(BkgError::InvalidContract{contract_id:c.id.to_string(),reason:"expired".into()}); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_contracts::CausalContractBuilder;
    use bkg_core::{Capability, RealmId};
    fn all()->Vec<Capability>{Capability::all().to_vec()}
    #[test] fn valid(){let c=CausalContractBuilder::new(RealmId::Telum,RealmId::Causa).require(Capability::CapsuleRead).build();CausalContractValidator.check(&c,&all(),LogicalTimestamp::ZERO).unwrap();}
    #[test] fn missing_cap(){let c=CausalContractBuilder::new(RealmId::Telum,RealmId::Causa).require(Capability::PolicyOverride).build();assert!(CausalContractValidator.check(&c,&[],LogicalTimestamp::ZERO).is_err());}
}
