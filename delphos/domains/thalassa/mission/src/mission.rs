use serde::{Deserialize, Serialize};
use chrono::{DateTime,Utc};
fn new_id(p:&str)->String{format!("{}-{}",&uuid::Uuid::new_v4().to_string()[..8].to_uppercase(),p)}
#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]pub struct MissionId(pub String);
impl MissionId{pub fn new()->Self{Self(new_id("M"))}}
impl std::fmt::Display for MissionId{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.0)}}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]#[serde(rename_all="snake_case")]
pub enum MissionStatus{#[default]Planning,Active,Blocked,Complete,Archived}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Mission{pub id:MissionId,pub title:String,pub description:Option<String>,pub status:MissionStatus,pub milestone_ids:Vec<String>,pub created_at:DateTime<Utc>}
impl Mission{pub fn new(title:impl Into<String>)->Self{Self{id:MissionId::new(),title:title.into(),description:None,status:Default::default(),milestone_ids:vec![],created_at:Utc::now()}}}

#[allow(clippy::derivable_impls)]
impl Default for MissionId{fn default()->Self{Self::new()}}
