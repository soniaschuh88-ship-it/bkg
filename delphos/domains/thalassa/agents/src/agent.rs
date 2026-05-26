//! Agent identity types. Single source of truth for all agent IDs.
//! Maps to sandbox-agent AgentId but rebranded for BKG.

use serde::{Deserialize, Serialize};

/// Canonical agent identifier. Single source of truth.
///
/// Every session, event, and credential references exactly one AgentId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AgentId {
    /// Anthropic Claude Code
    Claude,
    /// OpenAI Codex
    Codex,
    /// OpenCode (open-source)
    Opencode,
    /// Amp coding agent
    Amp,
    /// Pi agent (from pi-free)
    Pi,
    /// Cursor AI editor
    Cursor,
    /// BKG-native mock agent for testing
    Mock,
}

impl AgentId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude    => "claude",
            Self::Codex     => "codex",
            Self::Opencode  => "opencode",
            Self::Amp       => "amp",
            Self::Pi        => "pi",
            Self::Cursor    => "cursor",
            Self::Mock      => "mock",
        }
    }

    /// Name of the binary on PATH.
    pub fn binary_name(self) -> &'static str {
        match self {
            Self::Claude    => "claude",
            Self::Codex     => "codex",
            Self::Opencode  => "opencode",
            Self::Amp       => "amp",
            Self::Pi        => "pi",
            Self::Cursor    => "cursor-agent",
            Self::Mock      => "bkg-mock-agent",
        }
    }

    /// Human-readable display name.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Claude    => "Claude Code",
            Self::Codex     => "Codex (OpenAI)",
            Self::Opencode  => "OpenCode",
            Self::Amp       => "Amp",
            Self::Pi        => "Pi",
            Self::Cursor    => "Cursor",
            Self::Mock      => "BKG Mock Agent",
        }
    }

    /// Default provider (used to resolve API keys via bkg-providers).
    pub fn default_provider(self) -> &'static str {
        match self {
            Self::Claude    => "anthropic",
            Self::Codex     => "openai",
            Self::Opencode  => "openrouter",
            Self::Amp       => "anthropic",
            Self::Pi        => "kilo",
            Self::Cursor    => "anthropic",
            Self::Mock      => "ollama",
        }
    }

    /// Parse from string (lowercase).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "claude"   => Some(Self::Claude),
            "codex"    => Some(Self::Codex),
            "opencode" => Some(Self::Opencode),
            "amp"      => Some(Self::Amp),
            "pi"       => Some(Self::Pi),
            "cursor"   => Some(Self::Cursor),
            "mock"     => Some(Self::Mock),
            _ => None,
        }
    }

    pub fn all() -> &'static [AgentId] {
        &[Self::Claude, Self::Codex, Self::Opencode, Self::Amp, Self::Pi, Self::Cursor, Self::Mock]
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Agent execution mode.
///
/// Maps to sandbox-agent modes but extended with BKG-native supervisory modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    /// Default mode — agent runs with its own defaults.
    #[default]
    Default,
    /// Bypass mode — disables permission prompts (where supported).
    Bypass,
    /// Plan mode — agent plans before executing (Codex, Claude).
    PlanMode,
    /// BKG supervised — Plan→Review→Execute workflow gates enforced.
    BkgSupervised,
}

impl AgentMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default       => "default",
            Self::Bypass        => "bypass",
            Self::PlanMode      => "plan_mode",
            Self::BkgSupervised => "bkg_supervised",
        }
    }
}

/// Static metadata about an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub display_name: String,
    pub binary_name: String,
    pub default_provider: String,
    pub supported_modes: Vec<AgentMode>,
    pub supports_streaming: bool,
    pub supports_permissions: bool,
    pub supports_file_ops: bool,
    pub supports_images: bool,
}

impl AgentInfo {
    pub fn for_agent(id: AgentId) -> Self {
        let (modes, streaming, permissions, file_ops, images) = match id {
            AgentId::Claude   => (vec![AgentMode::Default, AgentMode::Bypass, AgentMode::BkgSupervised], true, true, true, false),
            AgentId::Codex    => (vec![AgentMode::Default, AgentMode::PlanMode, AgentMode::BkgSupervised], true, false, true, true),
            AgentId::Opencode => (vec![AgentMode::Default, AgentMode::BkgSupervised], true, false, true, false),
            AgentId::Amp      => (vec![AgentMode::Default, AgentMode::Bypass], true, true, true, false),
            AgentId::Pi       => (vec![AgentMode::Default, AgentMode::BkgSupervised], true, false, false, true),
            AgentId::Cursor   => (vec![AgentMode::Default], true, false, true, false),
            AgentId::Mock     => (vec![AgentMode::Default, AgentMode::Bypass, AgentMode::PlanMode, AgentMode::BkgSupervised], true, true, true, true),
        };
        Self {
            id,
            display_name: id.display_name().into(),
            binary_name: id.binary_name().into(),
            default_provider: id.default_provider().into(),
            supported_modes: modes,
            supports_streaming: streaming,
            supports_permissions: permissions,
            supports_file_ops: file_ops,
            supports_images: images,
        }
    }

    pub fn all() -> Vec<Self> {
        AgentId::all().iter().map(|&id| Self::for_agent(id)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn roundtrip_str() { for id in AgentId::all() { assert_eq!(AgentId::parse(id.as_str()), Some(*id)); } }
    #[test] fn display() { assert_eq!(AgentId::Claude.to_string(), "claude"); }
    #[test] fn all_have_info() { assert_eq!(AgentInfo::all().len(), 7); }
    #[test] fn supervised_mode() { let info = AgentInfo::for_agent(AgentId::Claude); assert!(info.supported_modes.contains(&AgentMode::BkgSupervised)); }
}