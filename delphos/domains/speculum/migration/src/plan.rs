use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MigrationStep{pub id:String,pub crate_name:String,pub from_version:u32,pub to_version:u32,pub description:String,pub transformer_id:Option<String>}
impl MigrationStep{pub fn new(crate_name:impl Into<String>,from:u32,to:u32,desc:impl Into<String>)->Self{Self{id:uuid::Uuid::new_v4().to_string(),crate_name:crate_name.into(),from_version:from,to_version:to,description:desc.into(),transformer_id:None}}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MigrationPlan{pub id:String,pub steps:Vec<MigrationStep>,pub created_at:DateTime<Utc>}
impl MigrationPlan{
    pub fn new(steps:Vec<MigrationStep>)->Self{Self{id:uuid::Uuid::new_v4().to_string(),steps,created_at:Utc::now()}}
    pub fn step_count(&self)->usize{self.steps.len()}
    pub fn affects_crate(&self,name:&str)->bool{self.steps.iter().any(|s|s.crate_name==name)}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn plan(){let p=MigrationPlan::new(vec![MigrationStep::new("bkg-task",1,2,"add priority field")]);assert_eq!(p.step_count(),1);assert!(p.affects_crate("bkg-task"));assert!(!p.affects_crate("bkg-mesh"));}
}
