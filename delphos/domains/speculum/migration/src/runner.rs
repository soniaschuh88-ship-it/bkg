use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
use crate::{plan::MigrationPlan,version_map::VersionMap};

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum MigrationOutcome{Applied,Skipped{reason:String},Failed{reason:String}}
impl MigrationOutcome{pub fn is_ok(&self)->bool{!matches!(self,Self::Failed{..})}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MigrationResult{pub step_id:String,pub outcome:MigrationOutcome,pub executed_at:DateTime<Utc>}

pub struct MigrationRunner{pub version_map:VersionMap}
impl MigrationRunner{
    pub fn new(vm:VersionMap)->Self{Self{version_map:vm}}
    pub fn run_plan(&mut self,plan:&MigrationPlan)->Vec<MigrationResult>{
        let mut results=Vec::new();
        for step in &plan.steps{
            let current=self.version_map.get(&step.crate_name);
            let outcome=if current>=step.to_version{
                MigrationOutcome::Skipped{reason:format!("already at v{}",current)}
            } else if current!=step.from_version{
                MigrationOutcome::Failed{reason:format!("expected v{}, found v{}",step.from_version,current)}
            } else {
                self.version_map.set(&step.crate_name,step.to_version);
                MigrationOutcome::Applied
            };
            results.push(MigrationResult{step_id:step.id.clone(),outcome,executed_at:Utc::now()});
        }
        results
    }
}
#[cfg(test)]
mod tests{use super::*;use crate::plan::MigrationStep;
    #[test] fn apply_migration(){
        let mut vm=VersionMap::new(); vm.set("bkg-task",1);
        let mut r=MigrationRunner::new(vm);
        let plan=MigrationPlan::new(vec![MigrationStep::new("bkg-task",1,2,"add field")]);
        let results=r.run_plan(&plan);
        assert_eq!(results[0].outcome,MigrationOutcome::Applied);
        assert_eq!(r.version_map.get("bkg-task"),2);
    }
    #[test] fn skip_if_ahead(){
        let mut vm=VersionMap::new(); vm.set("bkg-task",3);
        let mut r=MigrationRunner::new(vm);
        let plan=MigrationPlan::new(vec![MigrationStep::new("bkg-task",1,2,"old")]);
        let results=r.run_plan(&plan);
        assert!(matches!(results[0].outcome,MigrationOutcome::Skipped{..}));
    }
    #[test] fn fail_if_wrong_version(){
        let mut vm=VersionMap::new(); vm.set("bkg-task",0);
        let mut r=MigrationRunner::new(vm);
        let plan=MigrationPlan::new(vec![MigrationStep::new("bkg-task",1,2,"expects v1")]);
        let results=r.run_plan(&plan);
        assert!(matches!(results[0].outcome,MigrationOutcome::Failed{..}));
    }
}
