use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct Llm7Provider;
impl Llm7Provider{pub fn new()->Self{Self}}
impl Default for Llm7Provider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for Llm7Provider{
    fn id(&self)->&str{"llm7"}
    fn display_name(&self)->&str{"LLM7 Gateway (free)"}
    fn tier(&self)->&str{"free"}
    fn signup_url(&self)->Option<&str>{Some("https://llm7.io")}
    fn is_configured(&self)->bool{true}
    fn api_base_url(&self)->Option<String>{Some("https://api.llm7.io/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"default".into(),name:crate::enhancer::enhance_name("default","LLM7 Default"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:128000,max_tokens:4096,provider_id:"llm7".into()},ProviderModelConfig{id:"fast".into(),name:crate::enhancer::enhance_name("fast","LLM7 Fast"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:128000,max_tokens:4096,provider_id:"llm7".into()},])}
}