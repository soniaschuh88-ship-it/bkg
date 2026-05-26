use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct KiloProvider;
impl KiloProvider{pub fn new()->Self{Self}}
impl Default for KiloProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for KiloProvider{
    fn id(&self)->&str{"kilo"}
    fn display_name(&self)->&str{"Kilo (free tier)"}
    fn tier(&self)->&str{"free"}
    fn signup_url(&self)->Option<&str>{Some("https://kilo.codes")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("KILO_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.kilo.codes/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"claude-3-5-haiku-20241022".into(),name:crate::enhancer::enhance_name("claude-3-5-haiku-20241022","Claude 3.5 Haiku"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:200000,max_tokens:8192,provider_id:"kilo".into()},ProviderModelConfig{id:"claude-3-5-sonnet-20241022".into(),name:crate::enhancer::enhance_name("claude-3-5-sonnet-20241022","Claude 3.5 Sonnet"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(3_f64/1_000_000.0,15_f64/1_000_000.0),context_window:200000,max_tokens:8192,provider_id:"kilo".into()},])}
}