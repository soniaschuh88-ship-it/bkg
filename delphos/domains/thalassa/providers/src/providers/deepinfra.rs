use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct DeepinfraProvider;
impl DeepinfraProvider{pub fn new()->Self{Self}}
impl Default for DeepinfraProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for DeepinfraProvider{
    fn id(&self)->&str{"deepinfra"}
    fn display_name(&self)->&str{"DeepInfra (freemium)"}
    fn tier(&self)->&str{"freemium"}
    fn signup_url(&self)->Option<&str>{Some("https://deepinfra.com")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("DEEPINFRA_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.deepinfra.com/v1/openai".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"meta-llama/Meta-Llama-3.3-70B-Instruct".into(),name:crate::enhancer::enhance_name("meta-llama/Meta-Llama-3.3-70B-Instruct","Llama 3.3 70B"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(23_f64/1_000_000.0,23_f64/1_000_000.0),context_window:131072,max_tokens:32768,provider_id:"deepinfra".into()},ProviderModelConfig{id:"deepseek-ai/DeepSeek-R1".into(),name:crate::enhancer::enhance_name("deepseek-ai/DeepSeek-R1","DeepSeek R1"),reasoning:true,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(140_f64/1_000_000.0,140_f64/1_000_000.0),context_window:163840,max_tokens:8192,provider_id:"deepinfra".into()},])}
}