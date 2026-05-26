//! Agent credential extraction. Single source of truth.
//! Ported from sandbox-agent agent-credentials + agent-management/credentials.rs.
//!
//! Fallback chain (same as bkg-providers):
//!   1. User's own key (from BKG user config)
//!   2. Admin global key
//!   3. Environment variable
//!   4. Local agent config file (e.g. ~/.claude/credentials.json)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::agent::AgentId;

/// Where a credential came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSource {
    /// From BKG user config (highest priority)
    BkgUser,
    /// From BKG admin global config
    BkgAdmin,
    /// From environment variable
    Environment(String),
    /// From agent's own local config file
    AgentConfig(String),
    /// Anonymous (no key required — Kilo, LLM7)
    Anonymous,
    /// Not available
    NotConfigured,
}

impl CredentialSource {
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::NotConfigured)
    }
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BkgUser              => write!(f, "bkg-user"),
            Self::BkgAdmin             => write!(f, "bkg-admin"),
            Self::Environment(v)       => write!(f, "env:{v}"),
            Self::AgentConfig(p)       => write!(f, "config:{p}"),
            Self::Anonymous            => write!(f, "anonymous"),
            Self::NotConfigured        => write!(f, "not-configured"),
        }
    }
}

/// Resolved credentials for one agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCredentials {
    pub agent_id: AgentId,
    pub api_key: Option<String>,
    pub source: CredentialSource,
    /// Extra env vars the agent needs (e.g. ANTHROPIC_API_KEY, OPENAI_API_KEY).
    pub env_vars: HashMap<String, String>,
}

impl AgentCredentials {
    pub fn is_configured(&self) -> bool { self.source.is_available() }

    /// Build the env map to inject into the agent process.
    pub fn to_env_map(&self) -> HashMap<String, String> {
        let mut env = self.env_vars.clone();
        if let (Some(key), Some(var)) = (&self.api_key, api_key_env_var(self.agent_id)) {
            env.insert(var.to_string(), key.clone());
        }
        env
    }
}

/// Options for credential resolution.
#[derive(Debug, Clone, Default)]
pub struct CredentialExtractionOptions {
    /// BKG user-stored key override (from user's provider config).
    pub user_key: Option<String>,
    /// BKG admin global key override.
    pub admin_key: Option<String>,
    /// Skip reading agent config files.
    pub skip_agent_config: bool,
}

/// Canonical env var name for each agent's API key.
fn api_key_env_var(id: AgentId) -> Option<&'static str> {
    match id {
        AgentId::Claude   => Some("ANTHROPIC_API_KEY"),
        AgentId::Codex    => Some("OPENAI_API_KEY"),
        AgentId::Opencode => Some("OPENROUTER_API_KEY"),
        AgentId::Amp      => Some("ANTHROPIC_API_KEY"),
        AgentId::Pi       => Some("KILO_API_KEY"),
        AgentId::Cursor   => Some("ANTHROPIC_API_KEY"),
        AgentId::Mock     => None,
    }
}

/// Resolve credentials for an agent using the BKG fallback chain.
pub fn resolve_credentials(agent_id: AgentId, opts: &CredentialExtractionOptions) -> AgentCredentials {
    // 1. BKG user key
    if let Some(key) = &opts.user_key {
        if !key.is_empty() {
            return AgentCredentials { agent_id, api_key: Some(key.clone()), source: CredentialSource::BkgUser, env_vars: HashMap::new() };
        }
    }

    // 2. BKG admin global key
    if let Some(key) = &opts.admin_key {
        if !key.is_empty() {
            return AgentCredentials { agent_id, api_key: Some(key.clone()), source: CredentialSource::BkgAdmin, env_vars: HashMap::new() };
        }
    }

    // 3. Environment variable
    if let Some(var) = api_key_env_var(agent_id) {
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() {
                return AgentCredentials { agent_id, api_key: Some(val), source: CredentialSource::Environment(var.to_string()), env_vars: HashMap::new() };
            }
        }
    }

    // 4. Agent local config files (best-effort)
    if !opts.skip_agent_config {
        if let Some(creds) = extract_from_agent_config(agent_id) {
            return creds;
        }
    }

    // 5. Mock agent — always available
    if agent_id == AgentId::Mock {
        return AgentCredentials { agent_id, api_key: None, source: CredentialSource::Anonymous, env_vars: HashMap::new() };
    }

    AgentCredentials { agent_id, api_key: None, source: CredentialSource::NotConfigured, env_vars: HashMap::new() }
}

/// Resolve credentials for all agents.
pub fn resolve_all_credentials(opts: &CredentialExtractionOptions) -> Vec<AgentCredentials> {
    AgentId::all().iter().map(|&id| resolve_credentials(id, opts)).collect()
}

/// Try to extract a key from the agent's own local config file.
fn extract_from_agent_config(agent_id: AgentId) -> Option<AgentCredentials> {
    let home = std::env::var("HOME").ok()?;

    let (config_path, key_field): (&str, &str) = match agent_id {
        AgentId::Claude   => (".claude/credentials.json", "api_key"),
        AgentId::Codex    => (".openai/config.json", "api_key"),
        AgentId::Pi       => (".pi/free.json", "kilo_api_key"),
        _ => return None,
    };

    let path = format!("{home}/{config_path}");
    let content = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let key = json.get(key_field)?.as_str()?.to_string();
    if key.is_empty() { return None; }

    Some(AgentCredentials {
        agent_id,
        api_key: Some(key),
        source: CredentialSource::AgentConfig(path),
        env_vars: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn no_key_returns_not_configured() {
        // Mock has no real env var requirement
        let creds = resolve_credentials(AgentId::Claude, &CredentialExtractionOptions { skip_agent_config: true, ..Default::default() });
        assert!(!creds.is_configured() || creds.source == CredentialSource::BkgUser);
    }
    #[test] fn mock_always_available() {
        let creds = resolve_credentials(AgentId::Mock, &Default::default());
        assert!(creds.is_configured());
        assert_eq!(creds.source, CredentialSource::Anonymous);
    }
    #[test] fn user_key_wins() {
        let opts = CredentialExtractionOptions { user_key: Some("sk-test".into()), ..Default::default() };
        let creds = resolve_credentials(AgentId::Claude, &opts);
        assert_eq!(creds.source, CredentialSource::BkgUser);
        assert_eq!(creds.api_key.as_deref(), Some("sk-test"));
    }
    #[test] fn env_map_built() {
        let creds = AgentCredentials { agent_id: AgentId::Claude, api_key: Some("key123".into()), source: CredentialSource::BkgUser, env_vars: HashMap::new() };
        let env = creds.to_env_map();
        assert_eq!(env.get("ANTHROPIC_API_KEY").map(|s| s.as_str()), Some("key123"));
    }
}