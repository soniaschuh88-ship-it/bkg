use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum VerificationStatus{Passed,Warning,Failed}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum CheckStatus{Passed,Warning,Failed}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CheckResult{pub name:String,pub status:CheckStatus,#[serde(default,skip_serializing_if="Option::is_none")]pub detail:Option<String>}
impl CheckResult{
    pub fn pass(n:impl Into<String>)->Self{Self{name:n.into(),status:CheckStatus::Passed,detail:None}}
    pub fn warn(n:impl Into<String>,d:impl Into<String>)->Self{Self{name:n.into(),status:CheckStatus::Warning,detail:Some(d.into())}}
    pub fn fail(n:impl Into<String>,d:impl Into<String>)->Self{Self{name:n.into(),status:CheckStatus::Failed,detail:Some(d.into())}}
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VerificationReport{pub status:VerificationStatus,pub component:String,pub checks:Vec<CheckResult>,pub run_at:chrono::DateTime<chrono::Utc>}
impl VerificationReport{
    pub fn new(c:impl Into<String>)->Self{Self{status:VerificationStatus::Passed,component:c.into(),checks:Vec::new(),run_at:chrono::Utc::now()}}
    pub fn record(&mut self,r:CheckResult){match r.status{CheckStatus::Failed=>self.status=VerificationStatus::Failed,CheckStatus::Warning if self.status==VerificationStatus::Passed=>self.status=VerificationStatus::Warning,_=>{}}self.checks.push(r);}
    pub fn is_passed(&self)->bool{self.status==VerificationStatus::Passed}
    pub fn failure_count(&self)->usize{self.checks.iter().filter(|c|c.status==CheckStatus::Failed).count()}
}
