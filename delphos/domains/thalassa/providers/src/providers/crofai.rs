use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct CrofaiProvider;
impl CrofaiProvider{pub fn new()->Self{Self}}
impl Default for CrofaiProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for CrofaiProvider{
    fn id(&self)->&str{"crofai"}
    fn display_name(&self)->&str{"CrofAI (free-named)"}
    fn tier(&self)->&str{"paid"}
    fn signup_url(&self)->Option<&str>{Some("https://crofai.com")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("CROFAI_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.crofai.com/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"crofai-free-tier".into(),name:crate::enhancer::enhance_name("crofai-free-tier","CrofAI Free Tier"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:8192,max_tokens:4096,provider_id:"crofai".into()},ProviderModelConfig{id:"crofai-pro".into(),name:crate::enhancer::enhance_name("crofai-pro","CrofAI Pro"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(1000_f64/1_000_000.0,2000_f64/1_000_000.0),context_window:32768,max_tokens:8192,provider_id:"crofai".into()},])}
}