use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct SambanovaProvider;
impl SambanovaProvider{pub fn new()->Self{Self}}
impl Default for SambanovaProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for SambanovaProvider{
    fn id(&self)->&str{"sambanova"}
    fn display_name(&self)->&str{"SambaNova Cloud (free)"}
    fn tier(&self)->&str{"free"}
    fn signup_url(&self)->Option<&str>{Some("https://cloud.sambanova.ai")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("SAMBANOVA_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.sambanova.ai/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"Meta-Llama-3.3-70B-Instruct".into(),name:crate::enhancer::enhance_name("Meta-Llama-3.3-70B-Instruct","Llama 3.3 70B"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:131072,max_tokens:16384,provider_id:"sambanova".into()},ProviderModelConfig{id:"DeepSeek-R1".into(),name:crate::enhancer::enhance_name("DeepSeek-R1","DeepSeek R1"),reasoning:true,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:32768,max_tokens:16384,provider_id:"sambanova".into()},ProviderModelConfig{id:"Qwen2.5-Coder-32B-Instruct".into(),name:crate::enhancer::enhance_name("Qwen2.5-Coder-32B-Instruct","Qwen 2.5 Coder 32B"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:32768,max_tokens:8192,provider_id:"sambanova".into()},])}
}