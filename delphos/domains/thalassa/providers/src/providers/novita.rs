use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct NovitaProvider;
impl NovitaProvider{pub fn new()->Self{Self}}
impl Default for NovitaProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for NovitaProvider{
    fn id(&self)->&str{"novita"}
    fn display_name(&self)->&str{"Novita AI (freemium)"}
    fn tier(&self)->&str{"freemium"}
    fn signup_url(&self)->Option<&str>{Some("https://novita.ai")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("NOVITA_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.novita.ai/v3/openai".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"meta-llama/llama-3.1-8b-instruct".into(),name:crate::enhancer::enhance_name("meta-llama/llama-3.1-8b-instruct","Llama 3.1 8B (free)"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:131072,max_tokens:8192,provider_id:"novita".into()},ProviderModelConfig{id:"deepseek/deepseek-r1".into(),name:crate::enhancer::enhance_name("deepseek/deepseek-r1","DeepSeek R1 (free)"),reasoning:true,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:163840,max_tokens:8192,provider_id:"novita".into()},])}
}