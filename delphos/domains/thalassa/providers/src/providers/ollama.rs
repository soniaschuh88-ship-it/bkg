use async_trait::async_trait;use bkg_core::BkgResult;use reqwest::Client;
use crate::{enhancer,fetch::fetch_json,registry::ProviderAdapter,types::{CostConfig,ModelInput,ProviderModelConfig}};
pub struct OllamaProvider{client:Client,pub base_url:String}
impl OllamaProvider{pub fn new()->Self{Self{client:Client::new(),base_url:std::env::var("OLLAMA_HOST").unwrap_or_else(|_|"http://localhost:11434".into())}}}
impl Default for OllamaProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for OllamaProvider{
    fn id(&self)->&str{"ollama"}fn display_name(&self)->&str{"Ollama (local)"}
    fn is_configured(&self)->bool{true}fn tier(&self)->&str{"private"}
    fn api_base_url(&self)->Option<String>{Some(format!("{}/v1",self.base_url))}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{
        let raw=fetch_json(&self.client,&format!("{}/api/tags",self.base_url),None).await.unwrap_or_else(|_|serde_json::json!({"models":[]}));
        Ok(raw["models"].as_array().cloned().unwrap_or_default().iter().filter_map(|m|{
            let id=m["name"].as_str()?;
            Some(ProviderModelConfig{id:id.into(),name:enhancer::enhance_name(id,id),reasoning:id.contains("deepseek-r")||id.contains("qwq"),input:vec![ModelInput::Text],cost:CostConfig::free(),context_window:8192,max_tokens:4096,provider_id:"ollama".into()})
        }).collect())
    }
}