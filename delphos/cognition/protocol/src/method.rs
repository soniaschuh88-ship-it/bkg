//! ACP method registry. Single source of truth for all BKG ACP methods.
//! Extension methods use _bkg/ prefix (replaces sandbox-agent _sandboxagent/).

use serde::{Deserialize, Serialize};

/// All methods in the BKG ACP namespace.
///
/// Standard methods follow JSON-RPC 2.0 conventions.
/// BKG extensions use the `_bkg/` prefix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpMethod {
    // ── Session management ──────────────────────────────────────────────
    /// Create a new agent session.
    SessionCreate,
    /// Send a message to a session.
    SessionSend,
    /// Stream events from a session (returns SSE stream).
    SessionStream,
    /// Pause an active session.
    SessionPause,
    /// Resume a paused session.
    SessionResume,
    /// Destroy a session and clean up resources.
    SessionDestroy,
    /// Respond to a permission request.
    SessionPermissionRespond,

    // ── Agent management ────────────────────────────────────────────────
    /// List all registered agents and their status.
    AgentList,
    /// Get detailed status for one agent.
    AgentStatus,
    /// Install an agent.
    AgentInstall,
    /// Check/extract credentials for an agent.
    AgentCredentials,

    // ── Process runtime (sandbox-agent port) ────────────────────────────
    /// Start a process in the sandbox.
    ProcessStart,
    /// List running processes.
    ProcessList,
    /// Stream output from a process.
    ProcessStream,
    /// Kill a process.
    ProcessKill,

    // ── File operations ─────────────────────────────────────────────────
    /// Read a file.
    FileRead,
    /// Write a file.
    FileWrite,
    /// List a directory.
    FileList,
    /// Delete a file or directory.
    FileDelete,

    // ── BKG extensions ─────────────────────────────────────────────────
    /// Get BKG system info.
    BkgInfo,
    /// Get health status.
    BkgHealth,
    /// Detach from a session (keep running in background).
    BkgSessionDetach,
    /// Replay events from a session from a given offset.
    BkgSessionReplay,
}

impl AcpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionCreate            => "session/create",
            Self::SessionSend              => "session/send",
            Self::SessionStream            => "session/stream",
            Self::SessionPause             => "session/pause",
            Self::SessionResume            => "session/resume",
            Self::SessionDestroy           => "session/destroy",
            Self::SessionPermissionRespond => "session/permission_respond",
            Self::AgentList                => "agent/list",
            Self::AgentStatus              => "agent/status",
            Self::AgentInstall             => "agent/install",
            Self::AgentCredentials         => "agent/credentials",
            Self::ProcessStart             => "process/start",
            Self::ProcessList              => "process/list",
            Self::ProcessStream            => "process/stream",
            Self::ProcessKill              => "process/kill",
            Self::FileRead                 => "file/read",
            Self::FileWrite                => "file/write",
            Self::FileList                 => "file/list",
            Self::FileDelete               => "file/delete",
            Self::BkgInfo                  => "_bkg/info",
            Self::BkgHealth                => "_bkg/health",
            Self::BkgSessionDetach         => "_bkg/session/detach",
            Self::BkgSessionReplay         => "_bkg/session/replay",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "session/create"             => Some(Self::SessionCreate),
            "session/send"               => Some(Self::SessionSend),
            "session/stream"             => Some(Self::SessionStream),
            "session/pause"              => Some(Self::SessionPause),
            "session/resume"             => Some(Self::SessionResume),
            "session/destroy"            => Some(Self::SessionDestroy),
            "session/permission_respond" => Some(Self::SessionPermissionRespond),
            "agent/list"                 => Some(Self::AgentList),
            "agent/status"               => Some(Self::AgentStatus),
            "agent/install"              => Some(Self::AgentInstall),
            "agent/credentials"          => Some(Self::AgentCredentials),
            "process/start"              => Some(Self::ProcessStart),
            "process/list"               => Some(Self::ProcessList),
            "process/stream"             => Some(Self::ProcessStream),
            "process/kill"               => Some(Self::ProcessKill),
            "file/read"                  => Some(Self::FileRead),
            "file/write"                 => Some(Self::FileWrite),
            "file/list"                  => Some(Self::FileList),
            "file/delete"                => Some(Self::FileDelete),
            "_bkg/info"                  => Some(Self::BkgInfo),
            "_bkg/health"                => Some(Self::BkgHealth),
            "_bkg/session/detach"        => Some(Self::BkgSessionDetach),
            "_bkg/session/replay"        => Some(Self::BkgSessionReplay),
            _ => None,
        }
    }
}

impl std::fmt::Display for AcpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn roundtrip() {
        for m in [AcpMethod::SessionCreate, AcpMethod::BkgInfo, AcpMethod::ProcessStart] {
            assert_eq!(AcpMethod::parse(m.as_str()), Some(m));
        }
    }
    #[test] fn unknown() { assert!(AcpMethod::parse("unknown/method").is_none()); }
    #[test] fn bkg_prefix() { assert!(AcpMethod::BkgInfo.as_str().starts_with("_bkg/")); }
}