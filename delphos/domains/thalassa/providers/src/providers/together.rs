use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct TogetherProvider;
impl TogetherProvider{pub fn new()->Self{Self}}
impl Default for TogetherProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for TogetherProvider{
    fn id(&self)->&str{"together"}
    fn display_name(&self)->&str{"Together AI (freemium)"}
    fn tier(&self)->&str{"freemium"}
    fn signup_url(&self)->Option<&str>{Some("https://api.together.ai")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("TOGETHER_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.together.xyz/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"meta-llama/Llama-3.3-70B-Instruct-Turbo".into(),name:crate::enhancer::enhance_name("meta-llama/Llama-3.3-70B-Instruct-Turbo","Llama 3.3 70B"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(88_f64/1_000_000.0,88_f64/1_000_000.0),context_window:131072,max_tokens:32768,provider_id:"together".into()},ProviderModelConfig{id:"deepseek-ai/DeepSeek-R1".into(),name:crate::enhancer::enhance_name("deepseek-ai/DeepSeek-R1","DeepSeek R1"),reasoning:true,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(750_f64/1_000_000.0,750_f64/1_000_000.0),context_window:163840,max_tokens:8192,provider_id:"together".into()},])}
}