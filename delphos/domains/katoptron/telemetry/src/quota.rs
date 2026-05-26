use std::collections::HashMap;use serde::{Deserialize,Serialize};use chrono::{DateTime,Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct QuotaState{pub provider_id:String,pub requests_used:u64,pub requests_limit:Option<u64>,pub tokens_used:u64,pub tokens_limit:Option<u64>,pub reset_at:Option<DateTime<Utc>>,pub last_updated:DateTime<Utc>}
impl QuotaState{
    pub fn new(id:impl Into<String>)->Self{Self{provider_id:id.into(),requests_used:0,requests_limit:None,tokens_used:0,tokens_limit:None,reset_at:None,last_updated:Utc::now()}}
    pub fn is_exhausted(&self)->bool{self.requests_limit.map(|l|self.requests_used>=l).unwrap_or(false)||self.tokens_limit.map(|l|self.tokens_used>=l).unwrap_or(false)}
    pub fn seconds_until_reset(&self)->Option<i64>{self.reset_at.map(|r|(r-Utc::now()).num_seconds().max(0))}
    pub fn record_call(&mut self,p:u32,c:u32){self.requests_used+=1;self.tokens_used+=(p+c)as u64;self.last_updated=Utc::now();}
    pub fn format_status(&self)->String{match self.requests_limit{Some(l)=>format!("{}: {}/{} req",self.provider_id,self.requests_used,l),None=>format!("{}: {} req",self.provider_id,self.requests_used)}}
}
pub struct QuotaMonitor{states:HashMap<String,QuotaState>,persist_path:Option<std::path::PathBuf>}
impl Default for QuotaMonitor{fn default()->Self{Self::new()}}
impl QuotaMonitor{
    pub fn new()->Self{Self{states:HashMap::new(),persist_path:None}}
    pub fn open(path:impl AsRef<std::path::Path>)->bkg_core::BkgResult<Self>{let path=path.as_ref().to_path_buf();let states=if path.exists(){std::fs::read_to_string(&path).ok().and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default()}else{HashMap::new()};Ok(Self{states,persist_path:Some(path)})}
    pub fn state(&self,id:&str)->Option<&QuotaState>{self.states.get(id)}
    pub fn state_mut(&mut self,id:&str)->&mut QuotaState{self.states.entry(id.to_string()).or_insert_with(||QuotaState::new(id))}
    pub fn record_call(&mut self,id:&str,p:u32,c:u32)->bkg_core::BkgResult<()>{self.state_mut(id).record_call(p,c);if let Some(ref path)=self.persist_path.clone(){if let Some(par)=path.parent(){std::fs::create_dir_all(par)?;}let j=serde_json::to_string_pretty(&self.states).map_err(bkg_core::BkgError::Serialisation)?;std::fs::write(path,j)?;}Ok(())}
    pub fn all_states(&self)->Vec<&QuotaState>{let mut v:Vec<_>=self.states.values().collect();v.sort_by(|a,b|a.provider_id.cmp(&b.provider_id));v}
}
#[cfg(test)] mod tests{use super::*;
    #[test] fn record(){let mut m=QuotaMonitor::new();m.record_call("n",100,50).unwrap();assert_eq!(m.state("n").unwrap().requests_used,1);}
    #[test] fn exhausted(){let mut s=QuotaState::new("p");s.requests_limit=Some(5);s.requests_used=5;assert!(s.is_exhausted());}
}