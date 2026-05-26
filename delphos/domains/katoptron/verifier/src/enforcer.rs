// bkg-verifier/enforcer — Single source of truth for permission checks.
// One module, one location. Every tool invocation passes through here.
use serde::{Deserialize, Serialize};
use bkg_core::{BkgError, BkgResult};

/// Effective permission level. ReadOnly < WorkspaceWrite < DangerFullAccess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode { ReadOnly, WorkspaceWrite, DangerFullAccess }

impl PermissionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
    pub fn allows_write(self) -> bool { self >= Self::WorkspaceWrite }
    pub fn allows_full_access(self) -> bool { self == Self::DangerFullAccess }
}
impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) }
}

/// Outcome of a pre-execution permission check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementResult {
    Allow,
    Deny { reason: String },
    Prompt { message: String },
}
impl EnforcementResult {
    pub fn is_allowed(&self) -> bool { matches!(self, Self::Allow) }
    pub fn is_denied(&self) -> bool { matches!(self, Self::Deny { .. }) }
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Deny { reason } | Self::Prompt { message: reason } => Some(reason),
            Self::Allow => None,
        }
    }
}

/// A structured permission check request.
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub tool_name: String,
    pub input: String,
    pub mode: PermissionMode,
    pub path: Option<String>,
}
impl PermissionRequest {
    pub fn new(tool_name: impl Into<String>, input: impl Into<String>, mode: PermissionMode) -> Self {
        let input = input.into();
        let path = serde_json::from_str::<serde_json::Value>(&input)
            .ok()
            .and_then(|v| v.get("path").and_then(|p| p.as_str()).map(String::from));
        Self { tool_name: tool_name.into(), input, mode, path }
    }
}

/// Evaluates tool permission requests. The Speculum realm's enforcement point.
#[derive(Debug, Default)]
pub struct PermissionEnforcer;
impl PermissionEnforcer {
    pub fn new() -> Self { Self }

    pub fn check(&self, req: &PermissionRequest) -> EnforcementResult {
        match req.mode {
            PermissionMode::ReadOnly => {
                if self.requires_write(&req.tool_name) {
                    return EnforcementResult::Deny {
                        reason: format!(
                            "'{}' requires workspace-write; current mode is read-only",
                            req.tool_name
                        ),
                    };
                }
            }
            PermissionMode::WorkspaceWrite => {
                if self.requires_full_access(&req.tool_name) {
                    return EnforcementResult::Prompt {
                        message: format!(
                            "'{}' requires danger-full-access; confirm before proceeding",
                            req.tool_name
                        ),
                    };
                }
            }
            PermissionMode::DangerFullAccess => {}
        }
        EnforcementResult::Allow
    }

    pub fn check_all(&self, requests: &[PermissionRequest]) -> BkgResult<()> {
        for req in requests {
            if let EnforcementResult::Deny { reason } = self.check(req) {
                return Err(BkgError::MissingCapability(
                    format!("permission denied for '{}': {reason}", req.tool_name)
                ));
            }
        }
        Ok(())
    }

    fn requires_write(&self, name: &str) -> bool {
        matches!(name,
            "bash" | "write_file" | "edit_file" | "delete_file" |
            "git_commit" | "git_push" | "git_merge")
    }
    fn requires_full_access(&self, name: &str) -> bool {
        matches!(name, "dangerously_allow_any" | "network_unrestricted")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req(tool: &str, mode: PermissionMode) -> PermissionRequest {
        PermissionRequest::new(tool, r#"{"command":"echo hi"}"#, mode)
    }
    #[test] fn read_only_blocks_bash() {
        let r = PermissionEnforcer::new().check(&req("bash", PermissionMode::ReadOnly));
        assert!(r.is_denied());
        assert!(r.reason().unwrap().contains("read-only"));
    }
    #[test] fn workspace_write_allows_bash() {
        assert!(PermissionEnforcer::new().check(&req("bash", PermissionMode::WorkspaceWrite)).is_allowed());
    }
    #[test] fn read_only_allows_read_file() {
        assert!(PermissionEnforcer::new().check(&req("read_file", PermissionMode::ReadOnly)).is_allowed());
    }
    #[test] fn danger_allows_all() {
        let e = PermissionEnforcer::new();
        assert!(e.check(&req("bash", PermissionMode::DangerFullAccess)).is_allowed());
        assert!(e.check(&req("dangerously_allow_any", PermissionMode::DangerFullAccess)).is_allowed());
    }
    #[test] fn workspace_write_prompts_danger_tool() {
        let r = PermissionEnforcer::new().check(&req("dangerously_allow_any", PermissionMode::WorkspaceWrite));
        assert!(matches!(r, EnforcementResult::Prompt { .. }));
    }
    #[test] fn check_all_stops_on_first_denial() {
        let e = PermissionEnforcer::new();
        let reqs = vec![
            req("read_file", PermissionMode::ReadOnly),
            req("bash", PermissionMode::ReadOnly),
        ];
        assert!(e.check_all(&reqs).is_err());
    }
    #[test] fn path_extracted() {
        let r = PermissionRequest::new("read_file", r#"{"path":"/src/main.rs"}"#, PermissionMode::ReadOnly);
        assert_eq!(r.path.as_deref(), Some("/src/main.rs"));
    }
    #[test] fn mode_ordering() {
        assert!(PermissionMode::WorkspaceWrite > PermissionMode::ReadOnly);
        assert!(PermissionMode::DangerFullAccess > PermissionMode::WorkspaceWrite);
    }
}
