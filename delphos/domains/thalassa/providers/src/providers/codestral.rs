use async_trait::async_trait;use bkg_core::BkgResult;
use crate::{registry::ProviderAdapter,types::{CostConfig,ProviderModelConfig}};
pub struct CodestralProvider;
impl CodestralProvider{pub fn new()->Self{Self}}
impl Default for CodestralProvider{fn default()->Self{Self::new()}}
#[async_trait]impl ProviderAdapter for CodestralProvider{
    fn id(&self)->&str{"codestral"}
    fn display_name(&self)->&str{"Codestral (free exp)"}
    fn tier(&self)->&str{"freemium"}
    fn signup_url(&self)->Option<&str>{Some("https://console.mistral.ai")}
    fn is_configured(&self)->bool{crate::fetch::resolve_key("CODESTRAL_API_KEY",None).is_some()}
    fn api_base_url(&self)->Option<String>{Some("https://codestral.mistral.ai/v1".to_string())}
    async fn fetch_models(&self)->BkgResult<Vec<ProviderModelConfig>>{Ok(vec![ProviderModelConfig{id:"codestral-latest".into(),name:crate::enhancer::enhance_name("codestral-latest","Codestral Latest"),reasoning:false,input:vec![crate::types::ModelInput::Text],cost:CostConfig::new(0_f64/1_000_000.0,0_f64/1_000_000.0),context_window:262144,max_tokens:32768,provider_id:"codestral".into()},])}
}