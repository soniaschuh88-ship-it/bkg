//! Agent status and health checks. Single source of truth.
//! Ported from sandbox-agent agent-management status checking.

use std::process::Command;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{agent::{AgentId, AgentInfo}, credentials::{AgentCredentials, CredentialExtractionOptions, resolve_credentials}};

/// Live status of one agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: AgentId,
    pub installed: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub credentials_available: bool,
    pub credential_source: String,
    pub checked_at: DateTime<Utc>,
}

impl AgentStatus {
    /// Check the status of one agent (non-blocking: runs `binary --version`).
    pub fn check(agent_id: AgentId, opts: &CredentialExtractionOptions) -> Self {
        let binary = agent_id.binary_name();
        let (installed, version, binary_path) = probe_binary(binary);
        let creds = resolve_credentials(agent_id, opts);

        Self {
            agent_id,
            installed,
            version,
            binary_path,
            credentials_available: creds.is_configured(),
            credential_source: creds.source.to_string(),
            checked_at: Utc::now(),
        }
    }

    pub fn is_ready(&self) -> bool { self.installed && self.credentials_available }
}

/// Aggregate status report for all agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusReport {
    pub agents: Vec<AgentStatus>,
    pub ready_count: usize,
    pub installed_count: usize,
    pub generated_at: DateTime<Utc>,
}

impl AgentStatusReport {
    pub fn collect(opts: &CredentialExtractionOptions) -> Self {
        let agents: Vec<AgentStatus> = AgentId::all().iter()
            .map(|&id| AgentStatus::check(id, opts))
            .collect();
        let ready_count = agents.iter().filter(|s| s.is_ready()).count();
        let installed_count = agents.iter().filter(|s| s.installed).count();
        Self { agents, ready_count, installed_count, generated_at: Utc::now() }
    }
}

/// Probe a binary on PATH. Returns (found, version, path).
fn probe_binary(name: &str) -> (bool, Option<String>, Option<String>) {
    // Try `which`
    let path = Command::new("which").arg(name).output().ok()
        .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None })
        .map(|s| s.trim().to_string());

    if path.is_none() { return (false, None, None); }

    // Try `--version`
    let version = Command::new(name).arg("--version").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().next().unwrap_or("").trim().to_string());

    (true, version, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn mock_check() {
        // Mock agent won't be installed in CI, but status check shouldn't panic
        let status = AgentStatus::check(AgentId::Mock, &Default::default());
        assert_eq!(status.agent_id, AgentId::Mock);
        // credentials_available is true for Mock (anonymous)
        assert!(status.credentials_available);
    }
    #[test] fn report_all_agents() {
        let report = AgentStatusReport::collect(&Default::default());
        assert_eq!(report.agents.len(), 7);
        assert!(report.generated_at <= Utc::now());
    }
}