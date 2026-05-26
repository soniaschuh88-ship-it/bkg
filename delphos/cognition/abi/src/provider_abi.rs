// provider_abi.rs — LLM request/response normalization.
// All providers are accessed via this ABI — no provider-specific code leaks into callers.
use serde::{Deserialize, Serialize};
use crate::envelope::AbiEnvelope;

/// Normalized LLM request (OpenAI-compatible superset).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequestPayload {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub max_tokens: u32,
    pub stream: bool,
    pub temperature: Option<f64>,
    pub system: Option<String>,
    pub tools: Option<serde_json::Value>,
    /// Which provider to route to (e.g. "anthropic", "openrouter").
    pub provider_id: Option<String>,
    /// User-level API key override.
    pub user_key: Option<String>,
}

/// Normalized LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponsePayload {
    pub model: String,
    pub provider_id: String,
    pub reply: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub stop_reason: Option<String>,
    pub cost_usd: f64,
    pub latency_ms: u64,
}

pub type LlmRequestEnvelope = AbiEnvelope<LlmRequestPayload>;
pub type LlmResponseEnvelope = AbiEnvelope<LlmResponsePayload>;