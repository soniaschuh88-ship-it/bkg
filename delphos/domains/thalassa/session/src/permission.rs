//! Permission request / response for human-in-the-loop flows.
//! Single source of truth for all agent permission handling.
//! Ported from sandbox-agent permission streaming + bkg-verifier PermissionEnforcer.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// An agent is asking for permission to use a tool or take an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub tool_name: String,
    pub description: String,
    pub input_preview: Option<String>,
    pub risk_level: RiskLevel,
    pub requested_at: DateTime<Utc>,
}

impl PermissionRequest {
    pub fn new(tool_name: impl Into<String>, description: impl Into<String>, risk: RiskLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tool_name: tool_name.into(),
            description: description.into(),
            input_preview: None,
            risk_level: risk,
            requested_at: Utc::now(),
        }
    }
}

/// Risk level of the requested action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel { Low, Medium, High, Critical }

/// User's response to a permission request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub request_id: String,
    pub granted: bool,
    pub reason: Option<String>,
    pub decided_at: DateTime<Utc>,
}

impl PermissionResponse {
    pub fn grant(request_id: impl Into<String>) -> Self {
        Self { request_id: request_id.into(), granted: true, reason: None, decided_at: Utc::now() }
    }
    pub fn deny(request_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { request_id: request_id.into(), granted: false, reason: Some(reason.into()), decided_at: Utc::now() }
    }
}

/// How to handle permission requests automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStrategy {
    /// Always prompt the user (human-in-the-loop).
    #[default]
    AlwaysPrompt,
    /// Auto-approve low and medium risk; prompt for high and critical.
    AutoApproveSafe,
    /// Auto-approve everything (bypass mode — use with caution).
    AutoApproveAll,
    /// Auto-deny everything.
    AutoDenyAll,
}

impl PermissionStrategy {
    pub fn would_auto_approve(&self, risk: RiskLevel) -> bool {
        match self {
            Self::AlwaysPrompt    => false,
            Self::AutoApproveSafe => matches!(risk, RiskLevel::Low | RiskLevel::Medium),
            Self::AutoApproveAll  => true,
            Self::AutoDenyAll     => false,
        }
    }
}