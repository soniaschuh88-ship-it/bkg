use serde::{Deserialize,Serialize};use chrono::{DateTime,Utc};
fn nid()->String{format!("{}-MS",&uuid::Uuid::new_v4().to_string()[..8].to_uppercase())}
#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]pub struct MilestoneId(pub String);
impl MilestoneId{pub fn new()->Self{Self(nid())}}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Milestone{pub id:MilestoneId,pub mission_id:String,pub title:String,pub slice_ids:Vec<String>,pub created_at:DateTime<Utc>}
impl Milestone{pub fn new(mission_id:impl Into<String>,title:impl Into<String>)->Self{Self{id:MilestoneId::new(),mission_id:mission_id.into(),title:title.into(),slice_ids:vec![],created_at:Utc::now()}}}

#[allow(clippy::derivable_impls)]
impl Default for MilestoneId{fn default()->Self{Self::new()}}
