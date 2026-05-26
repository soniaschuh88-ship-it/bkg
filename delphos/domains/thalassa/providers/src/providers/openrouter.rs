use async_trait::async_trait;use bkg_core::BkgResult;use reqwest::Client;
use crate::{enhancer,fetch::{fetch_json,resolve_key},registry::ProviderAdapter,types::{CostConfig,ModelInput,ProviderModelConfig}};
const BASE:&str="https://openrouter.ai/api/v1";
pub struct OpenRouterProvider{client:Client}
impl OpenRouterProvider{pub fn new()->Self{Self{client:Client::new()}}}
impl Default for OpenRouterProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for OpenRouterProvider{
    fn id(&self)->&str{"openrouter"}fn display_name(&self)->&str{"OpenRouter (freemium)"}
    fn tier(&self)->&str{"freemium"}fn signup_url(&self)->Option<&str>{Some("https://openrouter.ai")}
    fn is_configured(&self)->bool{resolve_key("OPENROUTER_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some(BASE.to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{
        let key=match resolve_key("OPENROUTER_API_KEY",None){Some(k)=>k,None=>return Ok(vec![])};
        let raw=fetch_json(&self.client,&format!("{BASE}/models"),Some(&key)).await.unwrap_or_else(|_|serde_json::json!({"data":[]}));
        Ok(raw["data"].as_array().cloned().unwrap_or_default().iter().filter_map(|m|{
            let id=m["id"].as_str()?;let name=m["name"].as_str().unwrap_or(id);
            let ctx=m["context_length"].as_u64().unwrap_or(8192);
            let inp:f64=m["pricing"]["prompt"].as_str().and_then(|s|s.parse().ok()).unwrap_or(0.0);
            let out:f64=m["pricing"]["completion"].as_str().and_then(|s|s.parse().ok()).unwrap_or(0.0);
            Some(ProviderModelConfig{id:id.into(),name:enhancer::enhance_name(id,name),reasoning:id.contains("r1"),input:vec![ModelInput::Text],cost:CostConfig::new(inp,out),context_window:ctx,max_tokens:4096,provider_id:"openrouter".into()})
        }).collect())
    }
}