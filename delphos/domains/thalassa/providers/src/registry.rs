use std::collections::HashMap;
use async_trait::async_trait;
use bkg_core::BkgResult;
use serde::Serialize;
use crate::{detection::apply_free_filter, toggle::{ProviderToggleState,ToggleMode}, types::ProviderModelConfig};

#[async_trait]
pub trait ProviderAdapter: Send+Sync {
    fn id(&self)->&str;
    fn display_name(&self)->&str;
    fn is_configured(&self)->bool;
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>;
    fn api_base_url(&self)->Option<String>{None}
    /// Provider tier for UI grouping
    fn tier(&self)->&str{"freemium"}
    /// Signup URL shown in onboarding
    fn signup_url(&self)->Option<&str>{None}
}

pub struct RegistryEntry {
    pub adapter: Box<dyn ProviderAdapter>,
    pub models_free: Vec<ProviderModelConfig>,
    pub models_all: Vec<ProviderModelConfig>,
    pub last_fetched: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ProviderRegistry { entries:HashMap<String,RegistryEntry>, toggle:ProviderToggleState }
impl Default for ProviderRegistry { fn default()->Self{Self::new()} }
impl ProviderRegistry {
    pub fn new()->Self{Self{entries:HashMap::new(),toggle:ProviderToggleState::new()}}
    pub fn default_populated()->Self {
        use crate::providers::*;
        let mut r=Self::new();
        r.register(Box::new(OllamaProvider::new()));
        r.register(Box::new(NvidiaProvider::new()));
        r.register(Box::new(OpenRouterProvider::new()));
        r.register(Box::new(SambanovaProvider::new()));
        r.register(Box::new(Llm7Provider::new()));
        r.register(Box::new(KiloProvider::new()));
        r.register(Box::new(ClineProvider::new()));
        r.register(Box::new(ZenmuxProvider::new()));
        r.register(Box::new(CrofaiProvider::new()));
        r.register(Box::new(CodestralProvider::new()));
        r.register(Box::new(DeepinfraProvider::new()));
        r.register(Box::new(TogetherProvider::new()));
        r.register(Box::new(NovitaProvider::new()));
        r
    }
    pub fn register(&mut self,a:Box<dyn ProviderAdapter>){
        let id=a.id().to_string();
        self.entries.insert(id,RegistryEntry{adapter:a,models_free:vec![],models_all:vec![],last_fetched:None});
    }
    pub fn provider_ids(&self)->Vec<&str>{let mut v:Vec<&str>=self.entries.keys().map(|s|s.as_str()).collect();v.sort();v}
    pub fn models_for(&self,id:&str)->Vec<&ProviderModelConfig>{
        match self.entries.get(id){None=>vec![],Some(e)=>match self.toggle.get(id){
            ToggleMode::FreeOnly=>e.models_free.iter().collect(),
            ToggleMode::ShowAll=>e.models_all.iter().collect()}}
    }
    pub fn all_models(&self)->Vec<&ProviderModelConfig>{
        let mut ids:Vec<&str>=self.entries.keys().map(|s|s.as_str()).collect();ids.sort();
        ids.into_iter().flat_map(|id|self.models_for(id)).collect()
    }
    pub async fn refresh(&mut self,id:&str)->BkgResult<usize>{
        let e=self.entries.get_mut(id).ok_or_else(||bkg_core::BkgError::Internal(format!("provider '{id}' not found")))?;
        let all=e.adapter.fetch_models().await?;
        let free=apply_free_filter(all.clone(),true);
        let n=all.len(); e.models_all=all; e.models_free=free; e.last_fetched=Some(chrono::Utc::now()); Ok(n)
    }
    pub async fn refresh_all(&mut self)->HashMap<String,BkgResult<usize>>{
        let ids:Vec<String>=self.entries.keys().cloned().collect();
        let mut out=HashMap::new();
        for id in ids{let r=self.refresh(&id).await;out.insert(id,r);}
        out
    }
    pub fn toggle_provider(&mut self,id:&str)->ToggleMode{self.toggle.toggle_provider(id)}
    pub fn set_global_toggle(&mut self,m:ToggleMode){self.toggle.set_global(m);}
    pub fn is_configured(&self,id:&str)->bool{self.entries.get(id).map(|e|e.adapter.is_configured()).unwrap_or(false)}
    pub fn summary(&self)->Vec<ProviderSummary>{
        let mut ids:Vec<&str>=self.entries.keys().map(|s|s.as_str()).collect();ids.sort();
        ids.iter().map(|id|{let e=&self.entries[*id];ProviderSummary{
            id:id.to_string(),display_name:e.adapter.display_name().to_string(),
            tier:e.adapter.tier().to_string(),configured:e.adapter.is_configured(),
            model_count_all:e.models_all.len(),model_count_free:e.models_free.len(),
            toggle_mode:self.toggle.get(id),api_base_url:e.adapter.api_base_url(),
            signup_url:e.adapter.signup_url().map(String::from),
        }}).collect()
    }
}
#[derive(Debug,Clone,Serialize)]
pub struct ProviderSummary {
    pub id:String, pub display_name:String, pub tier:String,
    pub configured:bool, pub model_count_all:usize, pub model_count_free:usize,
    pub toggle_mode:ToggleMode, pub api_base_url:Option<String>, pub signup_url:Option<String>,
}
#[cfg(test)] mod tests { use super::*;
    #[test] fn count(){assert_eq!(ProviderRegistry::default_populated().provider_ids().len(),13);}
    #[test] fn toggle(){let mut r=ProviderRegistry::default_populated();assert_eq!(r.toggle_provider("nvidia"),ToggleMode::ShowAll);}
    #[test] fn summary(){assert_eq!(ProviderRegistry::default_populated().summary().len(),13);}
}