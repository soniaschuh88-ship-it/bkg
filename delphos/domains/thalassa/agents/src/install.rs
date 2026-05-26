//! Agent installation. Single source of truth.
//! Ported from sandbox-agent InstallOptions / InstallResult.

use serde::{Deserialize, Serialize};
use crate::agent::AgentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallSource {
    /// npm/bun package
    Npm { package: String, version: Option<String> },
    /// Download from URL
    Url(String),
    /// System package manager (apt, brew, etc.)
    System { package: String },
    /// Already installed — no-op
    AlreadyInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallOptions {
    pub source: Option<InstallSource>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub agent_id: AgentId,
    pub success: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub error: Option<String>,
    pub was_already_installed: bool,
}

impl InstallResult {
    pub fn already_installed(id: AgentId, version: Option<String>, path: Option<String>) -> Self {
        Self { agent_id: id, success: true, version, binary_path: path, error: None, was_already_installed: true }
    }
    pub fn failed(id: AgentId, error: impl Into<String>) -> Self {
        Self { agent_id: id, success: false, version: None, binary_path: None, error: Some(error.into()), was_already_installed: false }
    }
}

/// Recommended install source for each agent.
pub fn default_install_source(id: AgentId) -> InstallSource {
    match id {
        AgentId::Claude   => InstallSource::Npm { package: "@anthropic-ai/claude-code".into(), version: None },
        AgentId::Codex    => InstallSource::Npm { package: "@openai/codex".into(), version: None },
        AgentId::Opencode => InstallSource::Npm { package: "opencode-ai".into(), version: None },
        AgentId::Amp      => InstallSource::Npm { package: "@ampcode/cli".into(), version: None },
        AgentId::Pi       => InstallSource::Npm { package: "@earendil-works/pi-coding-agent".into(), version: None },
        AgentId::Cursor   => InstallSource::Url("https://cursor.sh/install".into()),
        AgentId::Mock     => InstallSource::AlreadyInstalled,
    }
}
