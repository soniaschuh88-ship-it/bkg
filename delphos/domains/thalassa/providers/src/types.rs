use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="lowercase")]
pub enum ModelInput { Text, Image, Audio }
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize,Default)]
pub struct CostConfig { pub input:f64, pub output:f64, #[serde(default)] pub cache_read:f64, #[serde(default)] pub cache_write:f64 }
impl CostConfig {
    pub fn free()->Self{Self::default()}
    pub fn new(i:f64,o:f64)->Self{Self{input:i,output:o,..Default::default()}}
    pub fn is_free(&self)->bool{self.input==0.0&&self.output==0.0}
}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct ProviderModelConfig {
    pub id:String, pub name:String, pub reasoning:bool,
    pub input:Vec<ModelInput>, pub cost:CostConfig,
    pub context_window:u64, pub max_tokens:u64, pub provider_id:String,
}
impl ProviderModelConfig {
    pub fn new(id:impl Into<String>,name:impl Into<String>,pid:impl Into<String>)->Self {
        Self{id:id.into(),name:name.into(),reasoning:false,input:vec![ModelInput::Text],
             cost:CostConfig::free(),context_window:8192,max_tokens:4096,provider_id:pid.into()}
    }
    pub fn with_cost(mut self,i:f64,o:f64)->Self{self.cost=CostConfig::new(i/1_000_000.0,o/1_000_000.0);self}
    pub fn with_context(mut self,c:u64,m:u64)->Self{self.context_window=c;self.max_tokens=m;self}
    pub fn with_reasoning(mut self)->Self{self.reasoning=true;self}
    pub fn is_free(&self)->bool{self.cost.is_free()}
}
#[cfg(test)] mod tests { use super::*;
    #[test] fn free(){assert!(ProviderModelConfig::new("m","m","p").is_free());}
    #[test] fn paid(){assert!(!ProviderModelConfig::new("m","m","p").with_cost(3.0,15.0).is_free());}
}