use std::sync::Arc;
use axum::{extract::State, Json};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::state::{AppMode, AppState};

type S = Arc<RwLock<AppState>>;

#[derive(Serialize)]
pub struct ModeResponse { pub mode: &'static str }

#[derive(Deserialize)]
pub struct SetModeRequest { pub mode: String }

pub async fn get_mode(State(s): State<S>) -> Json<ModeResponse> {
    let s = s.read().await;
    Json(ModeResponse { mode: s.mode.as_str() })
}

pub async fn set_mode(State(s): State<S>, Json(req): Json<SetModeRequest>) -> Json<ModeResponse> {
    let mut s = s.write().await;
    s.mode = match req.mode.as_str() { "private" => AppMode::Private, _ => AppMode::Cloud };
    Json(ModeResponse { mode: s.mode.as_str() })
}