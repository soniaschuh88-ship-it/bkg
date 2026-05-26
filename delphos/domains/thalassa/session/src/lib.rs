//! # bkg-session  (domains/thalassa/session)
//!
//! Session lifecycle, universal event schema, and SSE streaming.
//!
//! **Rebranded from** sandbox-agent universal-agent-schema + session management.
//!
//! **Single source of truth — one module, one location.**
//!
//! ## Core concepts
//! - `Session` — one conversation with one agent
//! - `UniversalEvent` — normalized event wrapper, agent-agnostic
//! - `UniversalEventData` — Message | Started | Error | PermissionAsked | QuestionAsked
//! - `SessionManager` — in-memory session registry
//! - `EventStream` — SSE-compatible event iterator

pub mod event;
pub mod manager;
pub mod message;
pub mod permission;
pub mod session;
pub mod stream;

pub use event::{UniversalEvent, UniversalEventData};
pub use manager::{SessionManager, SessionRef};
pub use message::{UniversalMessage, UniversalMessagePart};
pub use permission::{PermissionRequest, PermissionResponse, PermissionStrategy};
pub use session::{BkgSession, SendMessageOptions, SessionConfig, SessionState};
pub use stream::{EventStream, SseEvent};