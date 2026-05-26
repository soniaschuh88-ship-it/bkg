use std::collections::HashMap;
use bkg_core::BkgError;
use crate::{feature::Feature,milestone::{Milestone,MilestoneId},mission::{Mission,MissionId},slice::{Slice,SliceId}};
#[derive(Debug,Default)]
pub struct MissionStore{missions:HashMap<String,Mission>,milestones:HashMap<String,Milestone>,slices:HashMap<String,Slice>,features:HashMap<String,Feature>}
impl MissionStore{
    pub fn new()->Self{Self::default()}
    pub fn create_mission(&mut self,title:impl Into<String>)->Mission{let m=Mission::new(title);self.missions.insert(m.id.0.clone(),m.clone());m}
    pub fn create_milestone(&mut self,mid:&MissionId,title:impl Into<String>)->bkg_core::BkgResult<Milestone>{let m=self.missions.get_mut(&mid.0).ok_or_else(||BkgError::Internal(format!("mission {}",mid)))?;let ms=Milestone::new(&mid.0,title);m.milestone_ids.push(ms.id.0.clone());self.milestones.insert(ms.id.0.clone(),ms.clone());Ok(ms)}
    pub fn create_slice(&mut self,msid:&MilestoneId,title:impl Into<String>)->bkg_core::BkgResult<Slice>{let ms=self.milestones.get_mut(&msid.0).ok_or_else(||BkgError::Internal(format!("milestone {}",msid.0)))?;let sl=Slice::new(&msid.0,title);ms.slice_ids.push(sl.id.0.clone());self.slices.insert(sl.id.0.clone(),sl.clone());Ok(sl)}
    pub fn create_feature(&mut self,slid:&SliceId,title:impl Into<String>)->bkg_core::BkgResult<Feature>{let sl=self.slices.get_mut(&slid.0).ok_or_else(||BkgError::Internal(format!("slice {}",slid.0)))?;let f=Feature::new(&slid.0,title);sl.feature_ids.push(f.id.0.clone());self.features.insert(f.id.0.clone(),f.clone());Ok(f)}
    pub fn mission(&self,id:&MissionId)->Option<&Mission>{self.missions.get(&id.0)}
    pub fn all_missions(&self)->Vec<&Mission>{self.missions.values().collect()}
    pub fn all_features(&self)->Vec<&Feature>{self.features.values().collect()}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn hierarchy(){let mut s=MissionStore::new();let m=s.create_mission("BKG v1");let ms=s.create_milestone(&m.id,"Core").unwrap();let sl=s.create_slice(&ms.id,"Events").unwrap();let f=s.create_feature(&sl.id,"Ledger").unwrap();assert!(f.is_retryable());assert_eq!(f.budget_remaining(),3);}
    #[test] fn fix_budget(){let mut s=MissionStore::new();let m=s.create_mission("x");let ms=s.create_milestone(&m.id,"x").unwrap();let sl=s.create_slice(&ms.id,"x").unwrap();let f=s.create_feature(&sl.id,"x").unwrap();assert_eq!(f.budget_remaining(),3);}
}