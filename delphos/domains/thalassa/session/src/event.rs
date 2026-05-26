//! Universal event schema. Single source of truth for all agent events.
//! Ported from sandbox-agent universal-agent-schema UniversalEvent.
//! Every agent's native events are normalized into this format.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bkg_agents::AgentId;
use crate::{message::UniversalMessage, permission::PermissionRequest};

/// Wrapper for every event emitted by any agent session.
/// Agent-agnostic — the same type is used regardless of which agent is running.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    /// Monotonically increasing offset within the session.
    pub id: u64,
    /// Wall clock time.
    pub timestamp: DateTime<Utc>,
    /// Session this event belongs to.
    pub session_id: String,
    /// Which agent produced this event.
    pub agent: AgentId,
    /// The event payload.
    pub data: UniversalEventData,
}

impl UniversalEvent {
    pub fn new(id: u64, session_id: impl Into<String>, agent: AgentId, data: UniversalEventData) -> Self {
        Self { id, timestamp: Utc::now(), session_id: session_id.into(), agent, data }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(&self.data, UniversalEventData::Finished { .. } | UniversalEventData::Error { .. })
    }
}

/// All possible event payloads. Normalized across all agents.
/// Ported from sandbox-agent UniversalEventData enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniversalEventData {
    /// Agent session started.
    Started { mode: Option<String> },

    /// A message (assistant reply, tool call, tool result, file, etc.).
    Message(UniversalMessage),

    /// Incremental text delta (streaming mode).
    Delta { text: String, part_index: usize },

    /// Agent is asking a yes/no question.
    QuestionAsked { question: String, options: Vec<String>, message_id: Option<String> },

    /// Agent is requesting permission to use a tool or perform an action.
    PermissionAsked(PermissionRequest),

    /// User answered a question.
    QuestionAnswered { answer: String, message_id: Option<String> },

    /// Permission was granted or denied.
    PermissionDecided { granted: bool, tool_name: Option<String> },

    /// Session finished successfully.
    Finished { reason: Option<String> },

    /// Agent session encountered an error.
    Error { message: String, code: Option<String>, recoverable: bool },

    /// Raw agent event (unparseable — preserved verbatim).
    Unknown { raw: serde_json::Value },
}

impl UniversalEventData {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Started { .. }          => "started",
            Self::Message(_)              => "message",
            Self::Delta { .. }            => "delta",
            Self::QuestionAsked { .. }    => "question_asked",
            Self::PermissionAsked(_)      => "permission_asked",
            Self::QuestionAnswered { .. } => "question_answered",
            Self::PermissionDecided { .. }=> "permission_decided",
            Self::Finished { .. }         => "finished",
            Self::Error { .. }            => "error",
            Self::Unknown { .. }          => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn event_type_names() {
        let ev = UniversalEventData::Started { mode: None };
        assert_eq!(ev.event_type(), "started");
    }
    #[test] fn terminal_events() {
        let ev = UniversalEvent::new(1, "s", AgentId::Mock, UniversalEventData::Finished { reason: None });
        assert!(ev.is_terminal());
        let ev2 = UniversalEvent::new(2, "s", AgentId::Mock, UniversalEventData::Delta { text: "hi".into(), part_index: 0 });
        assert!(!ev2.is_terminal());
    }
    #[test] fn serde_roundtrip() {
        let ev = UniversalEvent::new(1, "sess-1", AgentId::Claude,
            UniversalEventData::Error { message: "timeout".into(), code: Some("E001".into()), recoverable: true });
        let json = serde_json::to_string(&ev).unwrap();
        let back: UniversalEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 1);
    }
}