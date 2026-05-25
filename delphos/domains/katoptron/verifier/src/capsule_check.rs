use bkg_capsule::Capsule;
use bkg_core::Hash256;
use crate::report::{CheckResult,VerificationReport};
pub fn verify_capsule_chain(h:&[Capsule])->VerificationReport{
    let mut r=VerificationReport::new("capsule_chain");
    if h.is_empty(){r.record(CheckResult::warn("non_empty","empty"));return r;}
    let mut exp=Hash256::ZERO;
    for(i,c)in h.iter().enumerate(){
        if!c.verify_integrity(){r.record(CheckResult::fail("self_integrity",format!("v{} bad",c.version)));}
        if c.prev_hash!=exp{r.record(CheckResult::fail("chain_link",format!("v{} idx {i}",c.version)));}
        if c.version!=(i as u64+1){r.record(CheckResult::fail("version_order",format!("v{} at {i}",c.version)));}
        exp=c.integrity_hash;
    }
    if r.failure_count()==0&&r.checks.is_empty(){r.record(CheckResult::pass(format!("chain_{}_versions",h.len())));}
    r
}
#[cfg(test)]mod tests{use super::*;use bkg_capsule::CapsuleManager;use bkg_core::RealmId;
    fn build(n:usize)->Vec<Capsule>{let mut m=CapsuleManager::new();let c=m.create(RealmId::Causa,None,serde_json::json!({"v":0})).unwrap();let id=c.capsule_id;for i in 1..n{m.update(id,serde_json::json!({"v":i}),None).unwrap();}m.history(&id).iter().map(|c|(*c).clone()).collect()}
    #[test]fn valid(){assert!(verify_capsule_chain(&build(3)).is_passed());}
    #[test]fn tamper(){let mut h=build(3);h[1].state_snapshot=serde_json::json!({"t":1});assert!(!verify_capsule_chain(&h).is_passed());}
}
