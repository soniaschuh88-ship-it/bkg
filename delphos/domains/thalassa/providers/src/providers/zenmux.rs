use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct ZenmuxProvider;
impl ZenmuxProvider{pub fn new()->Self{Self}}
impl Default for ZenmuxProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for ZenmuxProvider{
    fn id(&self)->&str{"zenmux"}
    fn display_name(&self)->&str{"ZenMux AI (paid)"}
    fn tier(&self)->&str{"paid"}
    fn signup_url(&self)->Option<&str>{Some("https://zenmux.ai")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("ZENMUX_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.zenmux.ai/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"gpt-4o".into(),name:crate::enhancer::enhance_name("gpt-4o","GPT-4o"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(2500_f64/1_000_000.0,10000_f64/1_000_000.0),context_window:128000,max_tokens:16384,provider_id:"zenmux".into()},ProviderModelConfig{id:"claude-3-5-sonnet-20241022".into(),name:crate::enhancer::enhance_name("claude-3-5-sonnet-20241022","Claude 3.5 Sonnet"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(3000_f64/1_000_000.0,15000_f64/1_000_000.0),context_window:200000,max_tokens:8192,provider_id:"zenmux".into()},])}
}