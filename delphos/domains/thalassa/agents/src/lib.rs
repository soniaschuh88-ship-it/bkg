//! # bkg-agents  (domains/thalassa/agents)
//!
//! Agent identity, lifecycle, credential extraction, and installation.
//!
//! **Rebranded from** [sandbox-agent](https://github.com/rivet-dev/sandbox-agent)
//! `server/packages/agent-management` — fully integrated into BKG DELPHOS.
//!
//! **Single source of truth — one module, one location.**
//! Every agent-related concept lives here.
//!
//! ## Supported agents
//! | BKG ID | Upstream | Binary |
//! |--------|----------|--------|
//! | `claude` | Anthropic Claude Code | `claude` |
//! | `codex` | OpenAI Codex | `codex` |
//! | `opencode` | OpenCode | `opencode` |
//! | `amp` | Amp | `amp` |
//! | `pi` | Pi | `pi` |
//! | `cursor` | Cursor | `cursor-agent` |
//!
//! ## BKG-native agent modes
//! - **Autonomous**: full agent control via `bkg-session`
//! - **Supervised**: workflow gates via `bkg-workflow` (Plan→Review→Execute)
//! - **Observed**: read-only inspection via `bkg-telemetry`

pub mod agent;
pub mod credentials;
pub mod install;
pub mod status;

pub use agent::{AgentId, AgentInfo, AgentMode};
pub use credentials::{AgentCredentials, CredentialSource, resolve_credentials};
pub use install::{InstallOptions, InstallResult, InstallSource};
pub use status::{AgentStatus, AgentStatusReport};