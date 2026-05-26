use serde::{Deserialize,Serialize};use chrono::{DateTime,Utc};
fn nid()->String{format!("{}-SL",&uuid::Uuid::new_v4().to_string()[..8].to_uppercase())}
#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]pub struct SliceId(pub String);
impl SliceId{pub fn new()->Self{Self(nid())}}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Slice{pub id:SliceId,pub milestone_id:String,pub title:String,pub feature_ids:Vec<String>,pub created_at:DateTime<Utc>}
impl Slice{pub fn new(milestone_id:impl Into<String>,title:impl Into<String>)->Self{Self{id:SliceId::new(),milestone_id:milestone_id.into(),title:title.into(),feature_ids:vec![],created_at:Utc::now()}}}

#[allow(clippy::derivable_impls)]
impl Default for SliceId{fn default()->Self{Self::new()}}
