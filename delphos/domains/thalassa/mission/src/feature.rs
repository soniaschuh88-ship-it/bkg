use serde::{Deserialize,Serialize};use chrono::{DateTime,Utc};
fn nid()->String{format!("{}-F",&uuid::Uuid::new_v4().to_string()[..8].to_uppercase())}
#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]pub struct FeatureId(pub String);
#[allow(clippy::derivable_impls)]
impl Default for FeatureId{fn default()->Self{Self::new()}}
impl FeatureId{pub fn new()->Self{Self(nid())}}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]#[serde(rename_all="snake_case")]
pub enum FeatureStatus{#[default]Pending,InProgress,Done,Failed}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Feature{pub id:FeatureId,pub slice_id:String,pub title:String,pub acceptance_criteria:Vec<String>,pub task_id:Option<String>,pub status:FeatureStatus,pub fix_budget:u32,pub fix_attempts:u32,pub created_at:DateTime<Utc>}
impl Feature{
    pub fn new(slice_id:impl Into<String>,title:impl Into<String>)->Self{Self{id:FeatureId::new(),slice_id:slice_id.into(),title:title.into(),acceptance_criteria:vec![],task_id:None,status:Default::default(),fix_budget:3,fix_attempts:0,created_at:Utc::now()}}
    pub fn budget_remaining(&self)->u32{self.fix_budget.saturating_sub(self.fix_attempts)}
    pub fn is_retryable(&self)->bool{self.budget_remaining()>0&&self.status!=FeatureStatus::Done}
}