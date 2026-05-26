use bkg_core::{BkgError,BkgResult};
use reqwest::Client;
use std::time::Duration;
pub const DEFAULT_TIMEOUT_MS:u64=10_000;
pub async fn fetch_json(client:&Client,url:&str,bearer:Option<&str>)->BkgResult<serde_json::Value>{
    let mut r=client.get(url);
    if let Some(k)=bearer{r=r.header("authorization",format!("Bearer {k}"));}
    let resp=r.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS)).send().await
        .map_err(|e|BkgError::Internal(format!("fetch {url}:{e}")))?;
    if !resp.status().is_success(){
        let s=resp.status(); let b=resp.text().await.unwrap_or_default();
        return Err(BkgError::Internal(format!("HTTP {s} {url}:{b}")));
    }
    resp.json().await.map_err(|e|BkgError::Internal(format!("JSON {url}:{e}")))
}
pub fn resolve_key(env_var:&str,override_key:Option<&str>)->Option<String>{
    override_key.filter(|k|!k.is_empty()).map(String::from)
        .or_else(||std::env::var(env_var).ok().filter(|k|!k.is_empty()))
}