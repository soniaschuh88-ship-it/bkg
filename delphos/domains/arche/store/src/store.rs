use bkg_capsule::Capsule;
use bkg_core::{BkgResult,CapsuleId,Hash256};
pub trait StateStore: Send+Sync {
    fn save(&self,c:&Capsule)->BkgResult<()>;
    fn load_current(&self,id:&CapsuleId)->BkgResult<Option<Capsule>>;
    fn load_version(&self,id:&CapsuleId,v:u64)->BkgResult<Option<Capsule>>;
    fn load_history(&self,id:&CapsuleId)->BkgResult<Vec<Capsule>>;
    fn capsule_count(&self)->BkgResult<usize>;
    fn snapshot_hash(&self)->BkgResult<Hash256>;
}
