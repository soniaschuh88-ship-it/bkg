use std::sync::Arc;
use axum::{extract::State, Json};
use tokio::sync::RwLock;
use serde::Deserialize;
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

pub async fn get_globals(State(s): State<S>) -> Json<serde_json::Value> {
    let s = s.read().await;
    // Mask actual key values
    let masked: std::collections::HashMap<String,String> = s.globals.keys.iter()
        .map(|(k,v)| (k.clone(), if v.is_empty() {"".into()} else {"••••••••".into()}))
        .collect();
    Json(serde_json::json!({
        "keys": masked,
        "default_model": s.globals.default_model,
        "free_only": s.globals.free_only,
    }))
}

#[derive(Deserialize)]
pub struct SetGlobalsRequest {
    pub keys: Option<std::collections::HashMap<String,String>>,
    pub default_model: Option<String>,
    pub free_only: Option<bool>,
}

pub async fn set_globals(State(s): State<S>, Json(req): Json<SetGlobalsRequest>) -> Json<serde_json::Value> {
    let mut sw = s.write().await;
    if let Some(keys) = req.keys {
        for (pid, key) in keys { if !key.is_empty() && !key.starts_with('•') { sw.globals.keys.insert(pid, key); } }
    }
    if let Some(m) = req.default_model { sw.globals.default_model = Some(m); }
    if let Some(f) = req.free_only { sw.globals.free_only = f; }
    match sw.save_globals() {
        Ok(_) => Json(serde_json::json!({"ok": true})),
        Err(e) => Json(serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

#[derive(Deserialize)]
pub struct UpdateProviderKeysRequest { pub provider_id: String, pub key: String }

pub async fn update_provider_keys(State(s): State<S>, Json(req): Json<UpdateProviderKeysRequest>) -> Json<serde_json::Value> {
    let mut sw = s.write().await;
    if !req.key.is_empty() { sw.globals.keys.insert(req.provider_id.clone(), req.key); }
    else { sw.globals.keys.remove(&req.provider_id); }
    let _ = sw.save_globals();
    Json(serde_json::json!({"ok": true, "provider": req.provider_id}))
}