//! InferenceProxy — routes inference requests through bkg-providers fallback chain.
//! Single source of truth for all inference API calls.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// An inference request (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}
fn default_max_tokens() -> u32 { 4096 }

/// An inference response (normalized).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub model: String,
    pub reply: String,
    pub provider_id: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub stop_reason: Option<String>,
}

/// Routes inference requests through the BKG provider fallback chain.
pub struct InferenceProxy {
    client: reqwest::Client,
    user_provider_keys: HashMap<String, String>,
    admin_provider_keys: HashMap<String, String>,
}

impl InferenceProxy {
    pub fn new(user_keys: HashMap<String, String>, admin_keys: HashMap<String, String>) -> Self {
        Self { client: reqwest::Client::new(), user_provider_keys: user_keys, admin_provider_keys: admin_keys }
    }

    /// Resolve the API key and base URL for a model.
    fn resolve(&self, model: &str) -> (Option<String>, String) {
        let provider = model.split('/').next().unwrap_or(model);
        // Fallback chain
        let key = self.user_provider_keys.get(provider)
            .or_else(|| self.admin_provider_keys.get(provider))
            .map(|k| k.clone());
        let base = match provider {
            "anthropic"  => "https://api.anthropic.com".into(),
            "openai"     => "https://api.openai.com".into(),
            "openrouter" => "https://openrouter.ai/api".into(),
            "sambanova"  => "https://api.sambanova.ai".into(),
            "together"   => "https://api.together.xyz".into(),
            "deepinfra"  => "https://api.deepinfra.com/v1/openai".into(),
            "ollama"     => std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".into()),
            _            => format!("https://api.{provider}.com"),
        };
        (key, base)
    }

    pub async fn complete(&self, req: InferenceRequest) -> bkg_core::BkgResult<InferenceResponse> {
        let (key, base_url) = self.resolve(&req.model);
        let provider = req.model.split('/').next().unwrap_or("unknown").to_string();

        let body = serde_json::json!({
            "model": req.model,
            "messages": req.messages,
            "max_tokens": req.max_tokens,
        });

        let mut rb = self.client.post(format!("{base_url}/v1/chat/completions")).json(&body);
        if let Some(k) = key { rb = rb.header("authorization", format!("Bearer {k}")); }

        let resp = rb.send().await.map_err(|e| bkg_core::BkgError::Internal(format!("inference: {e}")))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(bkg_core::BkgError::Internal(format!("inference {s}: {b}")));
        }
        let raw: serde_json::Value = resp.json().await.map_err(|e| bkg_core::BkgError::Internal(format!("json: {e}")))?;
        let reply = raw["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        let in_tok = raw["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let out_tok = raw["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(InferenceResponse {
            model: req.model,
            reply,
            provider_id: provider,
            input_tokens: in_tok,
            output_tokens: out_tok,
            stop_reason: raw["choices"][0]["finish_reason"].as_str().map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn resolve_anthropic() {
        let proxy = InferenceProxy::new(HashMap::new(), HashMap::new());
        let (key, base) = proxy.resolve("anthropic/claude-3-5-haiku");
        assert!(key.is_none());
        assert!(base.contains("anthropic"));
    }
    #[test] fn user_key_wins() {
        let mut user = HashMap::new();
        user.insert("anthropic".to_string(), "sk-user".to_string());
        let proxy = InferenceProxy::new(user, HashMap::new());
        let (key, _) = proxy.resolve("anthropic/claude-3-5-sonnet");
        assert_eq!(key.as_deref(), Some("sk-user"));
    }
}