//! AgentBridge — spawns and communicates with agent processes.
//! Ported from sandbox-agent acp-proxy-runtime + process-runtime.
//! Single source of truth for all agent process lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use bkg_agents::{AgentId, AgentMode, credentials::{AgentCredentials, CredentialExtractionOptions, resolve_credentials}};
use bkg_session::{UniversalEvent, UniversalEventData, BkgSession};

/// Config for creating an agent bridge.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub agent_id: AgentId,
    pub mode: AgentMode,
    pub working_dir: Option<String>,
    pub extra_env: HashMap<String, String>,
    pub credential_opts: CredentialExtractionOptions,
}

impl BridgeConfig {
    pub fn for_agent(agent_id: AgentId) -> Self {
        Self { agent_id, mode: AgentMode::Default, working_dir: None, extra_env: HashMap::new(), credential_opts: Default::default() }
    }
    pub fn with_user_key(mut self, key: impl Into<String>) -> Self {
        self.credential_opts.user_key = Some(key.into()); self
    }
}

/// Events emitted by the bridge about the underlying process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeEvent {
    /// Process started. Contains PID.
    ProcessStarted { pid: u32 },
    /// Stdout line received.
    Stdout { line: String },
    /// Stderr line received.
    Stderr { line: String },
    /// Process exited.
    ProcessExited { exit_code: i32 },
    /// Failed to start.
    StartFailed { reason: String },
}

/// Bridge state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeState { Idle, Running, Stopped, Failed }

/// Manages one agent process and translates I/O to/from UniversalEvents.
pub struct AgentBridge {
    pub config: BridgeConfig,
    pub credentials: AgentCredentials,
    state: Arc<RwLock<BridgeState>>,
    pub started_at: Option<DateTime<Utc>>,
}

impl AgentBridge {
    pub fn new(config: BridgeConfig) -> Self {
        let creds = resolve_credentials(config.agent_id, &config.credential_opts);
        Self { config, credentials: creds, state: Arc::new(RwLock::new(BridgeState::Idle)), started_at: None }
    }

    pub async fn state(&self) -> BridgeState { *self.state.read().await }
    pub fn is_configured(&self) -> bool { self.credentials.is_configured() }

    /// Build the environment map for the agent process.
    pub fn build_env(&self) -> HashMap<String, String> {
        let mut env = std::env::vars().collect::<HashMap<_,_>>();
        env.extend(self.credentials.to_env_map());
        env.extend(self.config.extra_env.clone());
        if let AgentMode::BkgSupervised = self.config.mode {
            env.insert("BKG_SUPERVISED".into(), "1".into());
        }
        env
    }

    /// Translate a raw stdout line from the agent to a UniversalEvent.
    /// Each agent has different event formats; this is the normalization layer.
    pub fn translate_stdout(&self, line: &str) -> Option<UniversalEventData> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");

        match (self.config.agent_id, event_type) {
            // Claude Code events
            (AgentId::Claude, "assistant") => {
                let text = v["message"]["content"].as_array()
                    .and_then(|c| c.iter().find(|b| b["type"] == "text"))
                    .and_then(|b| b["text"].as_str())
                    .unwrap_or("")
                    .to_string();
                Some(UniversalEventData::Message(bkg_session::UniversalMessage::text("assistant", text)))
            }
            (AgentId::Claude, "system") if v["subtype"] == "init" => {
                Some(UniversalEventData::Started { mode: Some(self.config.mode.as_str().into()) })
            }
            (AgentId::Claude, "result") => {
                Some(UniversalEventData::Finished { reason: v["subtype"].as_str().map(String::from) })
            }
            // Codex events
            (AgentId::Codex, "message") => {
                let text = v["content"].as_str().unwrap_or("").to_string();
                Some(UniversalEventData::Message(bkg_session::UniversalMessage::text("assistant", text)))
            }
            (AgentId::Codex, "task_complete") => {
                Some(UniversalEventData::Finished { reason: Some("task_complete".into()) })
            }
            // OpenCode events
            (AgentId::Opencode, "message.part.text") | (AgentId::Opencode, "message") => {
                let text = v["content"].as_str().or_else(|| v["text"].as_str()).unwrap_or("").to_string();
                Some(UniversalEventData::Message(bkg_session::UniversalMessage::text("assistant", text)))
            }
            // Mock agent — pass through
            (AgentId::Mock, _) => {
                let text = v["text"].as_str().or_else(|| v["content"].as_str()).unwrap_or(line).to_string();
                Some(UniversalEventData::Message(bkg_session::UniversalMessage::text("assistant", text)))
            }
            // Unknown format — preserve raw
            _ => Some(UniversalEventData::Unknown { raw: v }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn bridge_created_for_mock() {
        let bridge = AgentBridge::new(BridgeConfig::for_agent(AgentId::Mock));
        assert!(bridge.is_configured()); // Mock is always configured
    }
    #[test] fn env_map_includes_creds() {
        let mut cfg = BridgeConfig::for_agent(AgentId::Claude);
        cfg.credential_opts.user_key = Some("sk-test".into());
        let bridge = AgentBridge::new(cfg);
        let env = bridge.build_env();
        assert_eq!(env.get("ANTHROPIC_API_KEY").map(|s| s.as_str()), Some("sk-test"));
    }
    #[test] fn translate_mock_stdout() {
        let bridge = AgentBridge::new(BridgeConfig::for_agent(AgentId::Mock));
        let ev = bridge.translate_stdout(r#"{"text":"hello BKG"}"#);
        assert!(matches!(ev, Some(UniversalEventData::Message(_))));
    }
    #[test] fn translate_unknown_returns_raw() {
        let bridge = AgentBridge::new(BridgeConfig::for_agent(AgentId::Codex));
        let ev = bridge.translate_stdout(r#"{"type":"random","data":42}"#);
        assert!(matches!(ev, Some(UniversalEventData::Unknown { .. })));
    }
    #[test] fn supervised_mode_env() {
        let mut cfg = BridgeConfig::for_agent(AgentId::Mock);
        cfg.mode = AgentMode::BkgSupervised;
        let bridge = AgentBridge::new(cfg);
        let env = bridge.build_env();
        assert_eq!(env.get("BKG_SUPERVISED").map(|s| s.as_str()), Some("1"));
    }
}