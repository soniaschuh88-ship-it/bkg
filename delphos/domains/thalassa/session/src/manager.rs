//! SessionManager — in-memory session registry. Single source of truth.
//! Ported from sandbox-agent SessionManager.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use bkg_core::{BkgError, BkgResult};
use crate::session::{BkgSession, SessionConfig, SessionSummary};

pub type SessionRef = Arc<BkgSession>;

pub struct SessionManager {
    sessions: RwLock<HashMap<String, SessionRef>>,
}

impl Default for SessionManager { fn default() -> Self { Self::new() } }

impl SessionManager {
    pub fn new() -> Self { Self { sessions: RwLock::new(HashMap::new()) } }

    pub async fn create(&self, id: impl Into<String>, config: SessionConfig) -> SessionRef {
        let id = id.into();
        let sess = Arc::new(BkgSession::new(id.clone(), config));
        self.sessions.write().await.insert(id, sess.clone());
        sess
    }

    pub async fn get(&self, id: &str) -> Option<SessionRef> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn require(&self, id: &str) -> BkgResult<SessionRef> {
        self.get(id).await.ok_or_else(|| BkgError::Internal(format!("session '{id}' not found")))
    }

    pub async fn destroy(&self, id: &str) -> bool {
        self.sessions.write().await.remove(id).is_some()
    }

    pub async fn list(&self) -> Vec<SessionSummary> {
        let sessions = self.sessions.read().await;
        let mut v: Vec<SessionSummary> = sessions.values().map(|s| s.to_summary()).collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }

    pub async fn count(&self) -> usize { self.sessions.read().await.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_agents::AgentId;
    #[tokio::test] async fn create_get_destroy() {
        let mgr = SessionManager::new();
        mgr.create("s1", SessionConfig::for_agent(AgentId::Mock)).await;
        assert!(mgr.get("s1").await.is_some());
        assert!(mgr.destroy("s1").await);
        assert!(mgr.get("s1").await.is_none());
    }
    #[tokio::test] async fn require_missing() {
        let mgr = SessionManager::new();
        assert!(mgr.require("missing").await.is_err());
    }
    #[tokio::test] async fn list_sorted() {
        let mgr = SessionManager::new();
        mgr.create("a", SessionConfig::for_agent(AgentId::Mock)).await;
        mgr.create("b", SessionConfig::for_agent(AgentId::Claude)).await;
        let list = mgr.list().await;
        assert_eq!(list.len(), 2);
    }
}