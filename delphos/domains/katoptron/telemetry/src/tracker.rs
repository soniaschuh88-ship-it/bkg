use std::collections::HashMap;use std::path::{Path,PathBuf};
use serde::{Deserialize,Serialize};use chrono::{DateTime,Utc};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ModelCallRecord{pub timestamp:DateTime<Utc>,pub provider_id:String,pub model_id:String,pub success:bool,pub latency_ms:u64,pub prompt_tokens:u32,pub completion_tokens:u32,pub total_tokens:u32,pub tokens_per_second:f64,pub cost_usd:f64,#[serde(default,skip_serializing_if="Option::is_none")]pub stop_reason:Option<String>,#[serde(default,skip_serializing_if="Option::is_none")]pub error:Option<String>}
impl ModelCallRecord{
    pub fn success(pid:impl Into<String>,mid:impl Into<String>,lat:u64,p:u32,c:u32,cost:f64,stop:Option<String>)->Self{let t=p+c;let tps=if lat>0{c as f64/(lat as f64/1000.0)}else{0.0};Self{timestamp:Utc::now(),provider_id:pid.into(),model_id:mid.into(),success:true,latency_ms:lat,prompt_tokens:p,completion_tokens:c,total_tokens:t,tokens_per_second:tps,cost_usd:cost,stop_reason:stop,error:None}}
    pub fn failure(pid:impl Into<String>,mid:impl Into<String>,lat:u64,err:impl Into<String>)->Self{Self{timestamp:Utc::now(),provider_id:pid.into(),model_id:mid.into(),success:false,latency_ms:lat,prompt_tokens:0,completion_tokens:0,total_tokens:0,tokens_per_second:0.0,cost_usd:0.0,stop_reason:None,error:Some(err.into())}}
}
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct ModelStats{pub model_id:String,pub provider_id:String,pub total_calls:u64,pub success_calls:u64,pub error_calls:u64,pub total_tokens:u64,pub total_latency_ms:u64,pub total_cost_usd:f64,pub recent_calls:Vec<ModelCallRecord>}
impl ModelStats{
    pub fn new(m:impl Into<String>,p:impl Into<String>)->Self{Self{model_id:m.into(),provider_id:p.into(),..Default::default()}}
    pub fn record(&mut self,c:ModelCallRecord){self.total_calls+=1;if c.success{self.success_calls+=1;}else{self.error_calls+=1;}self.total_tokens+=c.total_tokens as u64;self.total_latency_ms+=c.latency_ms;self.total_cost_usd+=c.cost_usd;self.recent_calls.push(c);if self.recent_calls.len()>50{self.recent_calls.remove(0);}}
    pub fn avg_latency_ms(&self)->f64{if self.total_calls==0{0.0}else{self.total_latency_ms as f64/self.total_calls as f64}}
    pub fn success_rate(&self)->f64{if self.total_calls==0{1.0}else{self.success_calls as f64/self.total_calls as f64}}
}
pub struct ModelTracker{stats:HashMap<String,ModelStats>,persist_path:Option<PathBuf>}
impl Default for ModelTracker{fn default()->Self{Self::new()}}
impl ModelTracker{
    pub fn new()->Self{Self{stats:HashMap::new(),persist_path:None}}
    pub fn open(path:impl AsRef<Path>)->bkg_core::BkgResult<Self>{let path=path.as_ref().to_path_buf();let stats=if path.exists(){std::fs::read_to_string(&path).ok().and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default()}else{HashMap::new()};Ok(Self{stats,persist_path:Some(path)})}
    fn key(p:&str,m:&str)->String{format!("{p}/{m}")}
    pub fn record(&mut self,c:ModelCallRecord)->bkg_core::BkgResult<()>{let k=Self::key(&c.provider_id,&c.model_id);self.stats.entry(k).or_insert_with(||ModelStats::new(&c.model_id,&c.provider_id)).record(c);if let Some(ref p)=self.persist_path.clone(){if let Some(par)=p.parent(){std::fs::create_dir_all(par)?;}let j=serde_json::to_string_pretty(&self.stats).map_err(bkg_core::BkgError::Serialisation)?;std::fs::write(p,j)?;}Ok(())}
    pub fn stats_for(&self,p:&str,m:&str)->Option<&ModelStats>{self.stats.get(&Self::key(p,m))}
    pub fn all_stats(&self)->Vec<&ModelStats>{let mut v:Vec<_>=self.stats.values().collect();v.sort_by_key(|s|std::cmp::Reverse(s.total_calls));v}
    pub fn stats_for_provider(&self,p:&str)->Vec<&ModelStats>{self.stats.values().filter(|s|s.provider_id==p).collect()}
    pub fn total_cost_usd(&self)->f64{self.stats.values().map(|s|s.total_cost_usd).sum()}
    pub fn clear(&mut self){self.stats.clear();}
}
#[cfg(test)] mod tests{use super::*;
    fn sc(p:&str,m:&str,l:u64,t:u32)->ModelCallRecord{ModelCallRecord::success(p,m,l,t/2,t/2,0.0,None)}
    #[test] fn basic(){let mut t=ModelTracker::new();t.record(sc("p","m",1000,200)).unwrap();assert_eq!(t.stats_for("p","m").unwrap().total_calls,1);}
    #[test] fn persist(){let p=std::env::temp_dir().join(format!("bkg_t_{}.json",uuid::Uuid::new_v4()));{let mut t=ModelTracker::open(&p).unwrap();t.record(sc("o","l3",500,100)).unwrap();}let t2=ModelTracker::open(&p).unwrap();assert!(t2.stats_for("o","l3").is_some());let _=std::fs::remove_file(&p);}
}