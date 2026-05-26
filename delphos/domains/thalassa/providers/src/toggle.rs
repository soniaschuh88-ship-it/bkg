use std::collections::HashMap;
use serde::{Deserialize, Serialize};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]
#[serde(rename_all="snake_case")]
pub enum ToggleMode { #[default] FreeOnly, ShowAll }
impl ToggleMode {
    pub fn as_str(self)->&'static str{match self{Self::FreeOnly=>"free-only",Self::ShowAll=>"show-all"}}
    pub fn is_free_only(self)->bool{self==Self::FreeOnly}
    pub fn toggle(self)->Self{match self{Self::FreeOnly=>Self::ShowAll,Self::ShowAll=>Self::FreeOnly}}
}
impl std::fmt::Display for ToggleMode { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(self.as_str())} }
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize,Default)]
pub struct ProviderToggleState { pub global:ToggleMode, pub per_provider:HashMap<String,ToggleMode> }
impl ProviderToggleState {
    pub fn new()->Self{Self::default()}
    pub fn get(&self,id:&str)->ToggleMode{self.per_provider.get(id).copied().unwrap_or(self.global)}
    pub fn set(&mut self,id:impl Into<String>,m:ToggleMode){self.per_provider.insert(id.into(),m);}
    pub fn toggle_provider(&mut self,id:&str)->ToggleMode{let n=self.get(id).toggle();self.set(id,n);n}
    pub fn load_from_file(path:&std::path::Path)->Self {
        path.exists().then(||std::fs::read_to_string(path).ok())
            .flatten().and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn set_global(&mut self, m: ToggleMode) { self.global = m; }
    pub fn save_to_file(&self,path:&std::path::Path)->bkg_core::BkgResult<()> {
        if let Some(p)=path.parent(){std::fs::create_dir_all(p)?;}
        let j=serde_json::to_string_pretty(self).map_err(bkg_core::BkgError::Serialisation)?;
        std::fs::write(path,j)?; Ok(())
    }
}
#[cfg(test)] mod tests { use super::*;
    #[test] fn default_free(){assert_eq!(ProviderToggleState::new().get("x"),ToggleMode::FreeOnly);}
    #[test] fn toggle(){let mut s=ProviderToggleState::new();s.toggle_provider("k");assert_eq!(s.get("k"),ToggleMode::ShowAll);}
    #[test] fn serde(){let mut s=ProviderToggleState::new();s.set("n",ToggleMode::ShowAll);let j=serde_json::to_string(&s).unwrap();assert_eq!(s,serde_json::from_str(&j).unwrap());}
}