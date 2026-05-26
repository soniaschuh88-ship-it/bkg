use std::sync::Arc;
use axum::{extract::State, Json};
use axum::http::HeaderMap;
use tokio::sync::RwLock;
use serde::Deserialize;
use crate::{state::AppState, users::UserConfig};

type S = Arc<RwLock<AppState>>;

fn key_from_headers(headers: &HeaderMap) -> String {
    headers.get("x-bkg-key").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
}

pub async fn profile(State(s): State<S>, headers: HeaderMap) -> Json<serde_json::Value> {
    let key_id = key_from_headers(&headers);
    let s = s.read().await;
    let user = UserConfig::load(&s.data_dir, &key_id);
    Json(serde_json::json!({"key_id": user.key_id, "onboarded": user.onboarded}))
}

pub async fn get_user_keys(State(s): State<S>, headers: HeaderMap) -> Json<serde_json::Value> {
    let key_id = key_from_headers(&headers);
    let s = s.read().await;
    let user = UserConfig::load(&s.data_dir, &key_id);
    let masked: std::collections::HashMap<String,String> = user.provider_keys.iter()
        .map(|(k,v)| (k.clone(), if v.is_empty(){"".into()} else{"••••••••".into()}))
        .collect();
    Json(serde_json::json!({"keys": masked}))
}

#[derive(Deserialize)]
pub struct SetKeysRequest { pub keys: std::collections::HashMap<String,String> }

pub async fn set_user_keys(State(s): State<S>, headers: HeaderMap, Json(req): Json<SetKeysRequest>) -> Json<serde_json::Value> {
    let key_id = key_from_headers(&headers);
    let s = s.read().await;
    let mut user = UserConfig::load(&s.data_dir, &key_id);
    for (pid,key) in req.keys { if !key.is_empty() { user.provider_keys.insert(pid,key); } }
    match user.save(&s.data_dir) {
        Ok(_)  => Json(serde_json::json!({"ok": true})),
        Err(e) => Json(serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

pub async fn mark_onboarded(State(s): State<S>, headers: HeaderMap) -> Json<serde_json::Value> {
    let key_id = key_from_headers(&headers);
    let s = s.read().await;
    let mut user = UserConfig::load(&s.data_dir, &key_id);
    user.onboarded = true;
    let _ = user.save(&s.data_dir);
    Json(serde_json::json!({"ok": true}))
}

pub async fn self_register(State(s): State<S>, headers: HeaderMap) -> Json<serde_json::Value> {
    let ip = headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok()).unwrap_or("127.0.0.1")
        .split(',').next().unwrap_or("").trim().to_string();

    let mut sw = s.write().await;
    let now = chrono::Utc::now();
    let entry = sw.reg_rate.entry(ip.clone()).or_insert((0, now + chrono::Duration::hours(1)));
    if now > entry.1 { *entry = (0, now + chrono::Duration::hours(1)); }
    if entry.0 >= 3 {
        return Json(serde_json::json!({"error":"Rate limit: 3 registrations per hour"}));
    }
    entry.0 += 1;
    let user = UserConfig::create(&sw.data_dir);
    Json(serde_json::json!({"key": user.key_id, "ok": true}))
}