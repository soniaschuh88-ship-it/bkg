//! /agents/* — agent status, credentials, install endpoints.
//! Integrates bkg-agents into the Atlantean dashboard.

use std::sync::Arc;
use axum::{extract::{Path, State}, Json};
use tokio::sync::RwLock;
use serde::Deserialize;
use bkg_agents::{AgentId, AgentMode, AgentInfo,
    credentials::{CredentialExtractionOptions, resolve_all_credentials},
    status::AgentStatusReport};
use crate::state::AppState;

type S = Arc<RwLock<AppState>>;

/// GET /agents/list → all agents with status + capabilities
pub async fn list_agents(State(s): State<S>) -> Json<serde_json::Value> {
    let s = s.read().await;
    let opts = CredentialExtractionOptions {
        admin_key: None, user_key: None, skip_agent_config: false,
    };
    let report = AgentStatusReport::collect(&opts);
    let agents: Vec<_> = AgentInfo::all().iter().map(|info| {
        let status = report.agents.iter().find(|st| st.agent_id == info.id);
        serde_json::json!({
            "id": info.id.as_str(),
            "display_name": info.display_name,
            "default_provider": info.default_provider,
            "supported_modes": info.supported_modes.iter().map(|m| m.as_str()).collect::<Vec<_>>(),
            "supports_streaming": info.supports_streaming,
            "supports_permissions": info.supports_permissions,
            "supports_file_ops": info.supports_file_ops,
            "supports_images": info.supports_images,
            "installed": status.map(|s| s.installed).unwrap_or(false),
            "version": status.and_then(|s| s.version.as_deref()),
            "credentials_available": status.map(|s| s.credentials_available).unwrap_or(false),
            "credential_source": status.map(|s| s.credential_source.as_str()).unwrap_or("not_configured"),
            "ready": status.map(|s| s.is_ready()).unwrap_or(false),
        })
    }).collect();
    Json(serde_json::json!({
        "agents": agents,
        "ready_count": report.ready_count,
        "installed_count": report.installed_count,
    }))
}

/// GET /agents/:id/status
pub async fn agent_status(Path(id): Path<String>) -> Json<serde_json::Value> {
    let agent_id = match AgentId::parse(&id) {
        Some(a) => a,
        None => return Json(serde_json::json!({"error": format!("unknown agent: {id}")})),
    };
    let status = bkg_agents::status::AgentStatus::check(agent_id, &Default::default());
    Json(serde_json::json!({
        "agent_id": status.agent_id.as_str(),
        "installed": status.installed,
        "version": status.version,
        "binary_path": status.binary_path,
        "credentials_available": status.credentials_available,
        "credential_source": status.credential_source,
        "ready": status.is_ready(),
    }))
}

/// POST /agents/:id/credentials — set API key for an agent (via user BKG key)
#[derive(Deserialize)]
pub struct SetAgentCredentials { pub api_key: String }

pub async fn set_agent_credentials(
    State(s): State<S>,
    Path(id): Path<String>,
    Json(req): Json<SetAgentCredentials>,
) -> Json<serde_json::Value> {
    let agent_id = match AgentId::parse(&id) {
        Some(a) => a,
        None => return Json(serde_json::json!({"error": format!("unknown agent: {id}")})),
    };
    // Store under the agent's default provider ID in global keys
    let provider_id = agent_id.default_provider().to_string();
    let mut sw = s.write().await;
    sw.globals.keys.insert(provider_id.clone(), req.api_key);
    let _ = sw.save_globals();
    Json(serde_json::json!({"ok": true, "agent": id, "provider": provider_id}))
}