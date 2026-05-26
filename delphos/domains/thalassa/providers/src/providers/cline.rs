use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct ClineProvider;
impl ClineProvider{pub fn new()->Self{Self}}
impl Default for ClineProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for ClineProvider{
    fn id(&self)->&str{"cline"}
    fn display_name(&self)->&str{"Cline (free tier)"}
    fn tier(&self)->&str{"free"}
    fn signup_url(&self)->Option<&str>{Some("https://cline.bot")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("CLINE_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://api.cline.bot/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"claude-3-5-haiku-20241022".into(),name:crate::enhancer::enhance_name("claude-3-5-haiku-20241022","Claude 3.5 Haiku"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:200000,max_tokens:8192,provider_id:"cline".into()},])}
}