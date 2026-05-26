use std::sync::Arc;
use axum::{extract::{Path, State}, Json};
use tokio::sync::RwLock;
use serde::Deserialize;
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

pub async fn list(State(s): State<S>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let summary = s.registry.summary();
    Json(serde_json::json!({"providers": summary}))
}

pub async fn provider_models(State(s): State<S>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let models: Vec<_> = s.registry.models_for(&id).iter().map(|m| serde_json::json!({
        "id": m.id, "name": m.name, "free": m.is_free(),
        "reasoning": m.reasoning, "context_window": m.context_window,
        "input_cost": m.cost.input, "output_cost": m.cost.output,
    })).collect();
    Json(serde_json::json!({"provider": id, "models": models, "count": models.len()}))
}

#[derive(Deserialize)]
pub struct ProxyRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,

}

pub async fn proxy(State(s): State<S>, Json(req): Json<ProxyRequest>) -> Json<serde_json::Value> {
    // Determine provider from model id
    let provider_id = req.model.split('/').next().unwrap_or("unknown");
    let s_read = s.read().await;

    // Resolve API key using fallback chain
    let api_key = s_read.resolve_provider_key(provider_id, None);
    let base_url = s_read.registry.summary()
        .iter().find(|p| p.id == provider_id)
        .and_then(|p| p.api_base_url.clone())
        .unwrap_or_else(|| "https://api.openai.com".to_string());

    drop(s_read);

    // Forward to provider's OpenAI-compatible endpoint
    let client = reqwest::Client::new();
    let mut rb = client.post(format!("{base_url}/chat/completions"))
        .json(&serde_json::json!({
            "model": req.model, "messages": req.messages, "max_tokens": 4096,
        }));
    if let Some(k) = &api_key {
        rb = rb.header("authorization", format!("Bearer {k}"));
    }

    match rb.send().await {
        Ok(resp) if resp.status().is_success() => {
            let raw: serde_json::Value = resp.json().await.unwrap_or_default();
            let reply = raw["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            let in_tok = raw["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
            let out_tok = raw["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;
            // Record telemetry
            let call = bkg_telemetry::ModelCallRecord::success(
                provider_id, &req.model, 0, in_tok, out_tok, 0.0, Some("end_turn".into()));
            let mut sw = s.write().await;
            let _ = sw.tracker.record(call);
            let _ = sw.quota.record_call(provider_id, in_tok, out_tok);
            Json(serde_json::json!({"reply": reply, "provider": provider_id, "model": req.model}))
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Json(serde_json::json!({"error": format!("Provider {status}: {body}")}))
        }
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}