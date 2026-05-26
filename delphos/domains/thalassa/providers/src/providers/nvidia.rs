use async_trait::async_trait;use bkg_core::BkgResult;use reqwest::Client;
use crate::{enhancer,fetch::{fetch_json,resolve_key},registry::ProviderAdapter,types::{CostConfig,ModelInput,ProviderModelConfig}};
const BASE:&str="https://integrate.api.nvidia.com/v1";
pub struct NvidiaProvider{client:Client}
impl NvidiaProvider{pub fn new()->Self{Self{client:Client::new()}}}
impl Default for NvidiaProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for NvidiaProvider{
    fn id(&self)->&str{"nvidia"}fn display_name(&self)->&str{"NVIDIA NIM (freemium)"}
    fn tier(&self)->&str{"freemium"}fn signup_url(&self)->Option<&str>{Some("https://build.nvidia.com")}
    fn is_configured(&self)->bool{resolve_key("NVIDIA_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some(BASE.to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{
        let key=match resolve_key("NVIDIA_API_KEY",None){Some(k)=>k,None=>return Ok(vec![])};
        let raw=fetch_json(&self.client,&format!("{BASE}/models"),Some(&key)).await.unwrap_or_else(|_|serde_json::json!({"data":[]}));
        Ok(raw["data"].as_array().cloned().unwrap_or_default().iter().filter_map(|m|{
            let id=m["id"].as_str()?;
            if id.contains("embed")||id.contains("whisper"){return None;}
            Some(ProviderModelConfig{id:id.into(),name:enhancer::enhance_name(id,id),reasoning:id.contains("r1"),input:vec![ModelInput::Text],cost:CostConfig::free(),context_window:128_000,max_tokens:4096,provider_id:"nvidia".into()})
        }).collect())
    }
}