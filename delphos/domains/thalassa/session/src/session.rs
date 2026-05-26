//! BkgSession — one conversation with one agent. Single source of truth.
//! Ported from sandbox-agent SessionState + session lifecycle.

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_agents::{AgentId, AgentMode};
use crate::{event::{UniversalEvent, UniversalEventData}, message::UniversalMessage, permission::{PermissionStrategy, PermissionResponse}};

const EVENT_BROADCAST_CAP: usize = 512;

/// Current lifecycle state of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState { Pending, Running, AwaitingPermission, Paused, Finished, Error }

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub agent_id: AgentId,
    pub mode: AgentMode,
    pub system_prompt: Option<String>,
    pub permission_strategy: PermissionStrategy,
    /// BKG user key (for credential resolution)
    pub user_bkg_key: Option<String>,
    pub metadata: serde_json::Value,
}

impl SessionConfig {
    pub fn for_agent(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            mode: AgentMode::Default,
            system_prompt: None,
            permission_strategy: PermissionStrategy::default(),
            user_bkg_key: None,
            metadata: serde_json::Value::Null,
        }
    }
    pub fn with_mode(mut self, mode: AgentMode) -> Self { self.mode = mode; self }
    pub fn with_bkg_key(mut self, key: impl Into<String>) -> Self { self.user_bkg_key = Some(key.into()); self }
    pub fn supervised(mut self) -> Self { self.mode = AgentMode::BkgSupervised; self }
}

/// Options for sending a message.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SendMessageOptions {
    pub stream: bool,
    pub max_tokens: Option<u32>,
    pub attachments: Vec<String>,
}

/// One live session with an agent.
pub struct BkgSession {
    pub id: String,
    pub config: SessionConfig,
    state: Arc<RwLock<SessionState>>,
    events: Arc<RwLock<Vec<UniversalEvent>>>,
    event_tx: broadcast::Sender<UniversalEvent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Arc<RwLock<DateTime<Utc>>>,
}

impl BkgSession {
    pub fn new(id: impl Into<String>, config: SessionConfig) -> Self {
        let (tx, _) = broadcast::channel(EVENT_BROADCAST_CAP);
        Self {
            id: id.into(),
            config,
            state: Arc::new(RwLock::new(SessionState::Pending)),
            events: Arc::new(RwLock::new(Vec::new())),
            event_tx: tx,
            created_at: Utc::now(),
            updated_at: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Subscribe to live events (SSE-compatible).
    pub fn subscribe(&self) -> broadcast::Receiver<UniversalEvent> {
        self.event_tx.subscribe()
    }

    /// Get all events from an offset (for replay).
    pub async fn events_from(&self, offset: u64) -> Vec<UniversalEvent> {
        let ev = self.events.read().await;
        ev.iter().filter(|e| e.id >= offset).cloned().collect()
    }

    /// Emit an event into this session (used by the agent process bridge).
    pub async fn emit(&self, data: UniversalEventData) -> bkg_core::BkgResult<()> {
        let id = { let ev = self.events.read().await; ev.len() as u64 };
        let event = UniversalEvent::new(id, &self.id, self.config.agent_id, data);
        let is_terminal = event.is_terminal();
        {
            let mut ev = self.events.write().await;
            ev.push(event.clone());
        }
        *self.updated_at.write().await = Utc::now();
        let _ = self.event_tx.send(event);
        if is_terminal {
            let new_state = {
                let ev = self.events.read().await;
                if ev.iter().any(|e| matches!(&e.data, UniversalEventData::Error { .. })) {
                    SessionState::Error
                } else {
                    SessionState::Finished
                }
            };
            *self.state.write().await = new_state;
        }
        Ok(())
    }

    pub async fn state(&self) -> SessionState { *self.state.read().await }
    pub async fn event_count(&self) -> usize { self.events.read().await.len() }
    pub async fn set_state(&self, s: SessionState) { *self.state.write().await = s; }

    /// Respond to a pending permission request.
    pub async fn respond_permission(&self, response: PermissionResponse) -> bkg_core::BkgResult<()> {
        self.emit(UniversalEventData::PermissionDecided {
            granted: response.granted,
            tool_name: None,
        }).await?;
        if *self.state.read().await == SessionState::AwaitingPermission {
            *self.state.write().await = SessionState::Running;
        }
        Ok(())
    }

    pub fn to_summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id.clone(),
            agent_id: self.config.agent_id,
            mode: self.config.mode,
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub agent_id: AgentId,
    pub mode: AgentMode,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn create_and_emit() {
        let sess = BkgSession::new("s1", SessionConfig::for_agent(AgentId::Mock));
        assert_eq!(sess.state().await, SessionState::Pending);
        sess.emit(UniversalEventData::Started { mode: None }).await.unwrap();
        assert_eq!(sess.event_count().await, 1);
    }
    #[tokio::test] async fn terminal_event_sets_state() {
        let sess = BkgSession::new("s2", SessionConfig::for_agent(AgentId::Mock));
        sess.emit(UniversalEventData::Finished { reason: None }).await.unwrap();
        assert_eq!(sess.state().await, SessionState::Finished);
    }
    #[tokio::test] async fn broadcast_subscriber() {
        let sess = BkgSession::new("s3", SessionConfig::for_agent(AgentId::Mock));
        let mut rx = sess.subscribe();
        sess.emit(UniversalEventData::Delta { text: "hi".into(), part_index: 0 }).await.unwrap();
        let ev = rx.try_recv().unwrap();
        assert!(matches!(ev.data, UniversalEventData::Delta { .. }));
    }
    #[tokio::test] async fn events_from_offset() {
        let sess = BkgSession::new("s4", SessionConfig::for_agent(AgentId::Mock));
        for i in 0..5u64 { sess.emit(UniversalEventData::Delta { text: format!("t{i}"), part_index: 0 }).await.unwrap(); }
        let from2 = sess.events_from(2).await;
        assert_eq!(from2.len(), 3);
    }
}