//! /sessions/* — session create, send, stream, list, destroy.
//! Inspector backend: exposes BkgSession via REST + SSE.

use std::sync::Arc;
use axum::{extract::{Path, Query, State}, Json, response::{Sse, IntoResponse}};
use axum::response::sse::Event;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use futures_util::stream;
use serde::Deserialize;
use bkg_agents::{AgentId, AgentMode};
use bkg_session::{SessionConfig, UniversalEventData, SseEvent};
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

/// POST /sessions — create a new session
#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub session_id: Option<String>,
    pub agent_id: String,
    #[serde(default)]
    pub mode: Option<String>,
    pub system_prompt: Option<String>,
    pub user_bkg_key: Option<String>,
}

pub async fn create_session(State(s): State<S>, Json(req): Json<CreateSessionRequest>) -> Json<serde_json::Value> {
    let agent_id = match AgentId::parse(&req.agent_id) {
        Some(a) => a,
        None => return Json(serde_json::json!({"error": format!("unknown agent: {}", req.agent_id)})),
    };
    let mode = req.mode.as_deref().map(|m| match m {
        "bypass" => AgentMode::Bypass,
        "plan_mode" => AgentMode::PlanMode,
        "bkg_supervised" => AgentMode::BkgSupervised,
        _ => AgentMode::Default,
    }).unwrap_or_default();

    let mut cfg = SessionConfig::for_agent(agent_id).with_mode(mode);
    if let Some(k) = req.user_bkg_key { cfg = cfg.with_bkg_key(k); }
    if let Some(sp) = req.system_prompt { cfg.system_prompt = Some(sp); }

    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let s = s.read().await;
    let sess = s.session_manager.create(session_id.clone(), cfg).await;
    sess.emit(UniversalEventData::Started { mode: Some(mode.as_str().into()) }).await.unwrap_or(());

    Json(serde_json::json!({"session_id": session_id, "agent_id": req.agent_id, "mode": mode.as_str(), "ok": true}))
}

/// POST /sessions/:id/send
#[derive(Deserialize)]
pub struct SendRequest { pub text: String }

pub async fn send_message(State(s): State<S>, Path(id): Path<String>, Json(req): Json<SendRequest>) -> Json<serde_json::Value> {
    let s = s.read().await;
    match s.session_manager.get(&id).await {
        None => Json(serde_json::json!({"error": format!("session '{id}' not found")})),
        Some(sess) => {
            // Record the user message
            sess.emit(UniversalEventData::Message(bkg_session::UniversalMessage::text("user", &req.text))).await.unwrap_or(());
            // TODO: in future, forward to actual agent process via bkg-acp bridge
            // For now, emit a mock acknowledgement so the Inspector works
            sess.emit(UniversalEventData::Message(
                bkg_session::UniversalMessage::text("assistant",
                    format!("[BKG] Message received by session '{id}'. Agent bridge: connect via bkg-acp AgentBridge."))
            )).await.unwrap_or(());
            Json(serde_json::json!({"session_id": id, "ok": true}))
        }
    }
}

/// GET /sessions/:id/stream — SSE event stream
#[derive(Deserialize)]
pub struct StreamQuery { pub offset: Option<u64> }

pub async fn stream_session(State(s): State<S>, Path(id): Path<String>, Query(q): Query<StreamQuery>) -> impl IntoResponse {
    let s = s.read().await;
    match s.session_manager.get(&id).await {
        None => Sse::new(stream::once(async { Ok::<_,std::convert::Infallible>(Event::default().data("session not found")) })).into_response(),
        Some(sess) => {
            let rx = sess.subscribe();
            let offset = q.offset.unwrap_or(0);
            // First replay existing events from offset
            let past = sess.events_from(offset).await;
            let past_stream = stream::iter(past.into_iter().map(|ev| {
                let sse = SseEvent::from_universal(&ev);
                Ok::<_, std::convert::Infallible>(Event::default().event(&sse.event).data(&sse.data))
            }));
            // Then live
            let live = BroadcastStream::new(rx).filter_map(|r| r.ok()).map(|ev| {
                let sse = SseEvent::from_universal(&ev);
                Ok::<_, std::convert::Infallible>(Event::default().event(&sse.event).data(&sse.data))
            });
            Sse::new(past_stream.chain(live)).keep_alive(axum::response::sse::KeepAlive::default()).into_response()
        }
    }
}

/// GET /sessions — list all sessions
pub async fn list_sessions(State(s): State<S>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let sessions = s.session_manager.list().await;
    let rows: Vec<_> = sessions.iter().map(|s| serde_json::json!({
        "id": s.id, "agent_id": s.agent_id.as_str(), "mode": s.mode.as_str(),
        "created_at": s.created_at,
    })).collect();
    Json(serde_json::json!({"sessions": rows, "count": rows.len()}))
}

/// GET /sessions/:id — get session details + events
pub async fn get_session(State(s): State<S>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let s = s.read().await;
    match s.session_manager.get(&id).await {
        None => Json(serde_json::json!({"error": "session not found"})),
        Some(sess) => {
            let events = sess.events_from(0).await;
            Json(serde_json::json!({
                "id": sess.id,
                "agent_id": sess.config.agent_id.as_str(),
                "mode": sess.config.mode.as_str(),
                "state": format!("{:?}", sess.state().await).to_lowercase(),
                "event_count": events.len(),
                "events": events,
            }))
        }
    }
}

/// DELETE /sessions/:id
pub async fn destroy_session(State(s): State<S>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let destroyed = s.session_manager.destroy(&id).await;
    Json(serde_json::json!({"ok": destroyed, "session_id": id}))
}