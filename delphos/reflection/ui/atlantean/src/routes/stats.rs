use std::sync::Arc;
use axum::{extract::State, Json};
use tokio::sync::RwLock;
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

pub async fn stats(State(s): State<S>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let total_calls: u64 = s.tracker.all_stats().iter().map(|st| st.total_calls).sum();
    let total_cost: f64 = s.tracker.total_cost_usd();
    let model_count: usize = s.registry.all_models().len();
    Json(serde_json::json!({
        "provider_count": s.registry.provider_ids().len(),
        "model_count": model_count,
        "total_calls": total_calls,
        "total_cost": format!("{total_cost:.4}"),
        "mode": s.mode.as_str(),
    }))
}

pub async fn telemetry(State(s): State<S>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let models: Vec<_> = s.tracker.all_stats().iter().map(|st| serde_json::json!({
        "model": st.model_id, "provider": st.provider_id,
        "calls": st.total_calls,
        "success_rate": format!("{:.0}%", st.success_rate() * 100.0),
        "avg_latency_ms": st.avg_latency_ms().round(),
        "cost_usd": st.total_cost_usd,
    })).collect();
    Json(serde_json::json!({"models": models, "total_cost_usd": s.tracker.total_cost_usd()}))
}