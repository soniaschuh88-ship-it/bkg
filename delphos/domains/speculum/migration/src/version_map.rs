use std::collections::BTreeMap;
use serde::{Deserialize,Serialize};
/// Maps (crate_name → current schema version). Monotonically increasing.
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct VersionMap(BTreeMap<String,u32>);
impl VersionMap{
    pub fn new()->Self{Self::default()}
    pub fn get(&self,crate_name:&str)->u32{self.0.get(crate_name).copied().unwrap_or(0)}
    pub fn set(&mut self,crate_name:impl Into<String>,version:u32){self.0.insert(crate_name.into(),version);}
    pub fn bump(&mut self,crate_name:&str)->u32{let v=self.get(crate_name)+1;self.0.insert(crate_name.to_string(),v);v}
    pub fn all(&self)->Vec<(&str,u32)>{self.0.iter().map(|(k,&v)|(k.as_str(),v)).collect()}
}
#[cfg(test)]
mod tests{use super::*;
    #[test] fn get_set_bump(){let mut m=VersionMap::new();assert_eq!(m.get("bkg-task"),0);m.set("bkg-task",1);assert_eq!(m.get("bkg-task"),1);assert_eq!(m.bump("bkg-task"),2);}
}
