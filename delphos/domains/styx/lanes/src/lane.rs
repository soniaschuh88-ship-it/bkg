use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize,Hash,Default)]
#[serde(rename_all="snake_case")]
pub enum LaneClass { Background=0, #[default] Normal=1, High=2, Critical=3 }
impl LaneClass {
    pub fn as_str(self)->&'static str { match self { Self::Background=>"background",Self::Normal=>"normal",Self::High=>"high",Self::Critical=>"critical" } }
    pub fn capacity(self)->usize { match self { Self::Background=>512,Self::Normal=>256,Self::High=>128,Self::Critical=>32 } }
    pub fn latency_target_ms(self)->u64 { match self { Self::Background=>5000,Self::Normal=>500,Self::High=>100,Self::Critical=>10 } }
}
impl std::fmt::Display for LaneClass { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(self.as_str())} }
#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct Lane { pub source_realm:String, pub target_realm:String, pub class:LaneClass }
impl Lane {
    pub fn new(src:impl Into<String>,tgt:impl Into<String>,c:LaneClass)->Self{Self{source_realm:src.into(),target_realm:tgt.into(),class:c}}
    pub fn key(&self)->String{format!("{}→{}:{}",self.source_realm,self.target_realm,self.class)}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn priority_order(){assert!(LaneClass::Critical>LaneClass::Normal);}
    #[test] fn capacity(){assert!(LaneClass::Critical.capacity()<LaneClass::Normal.capacity());}
    #[test] fn key(){let l=Lane::new("telum","katoptron",LaneClass::High);assert!(l.key().contains("telum"));}
}
