//! Universal message schema. Single source of truth.
//! Ported from sandbox-agent UniversalMessage + UniversalMessagePart.

use serde::{Deserialize, Serialize};

/// A parsed or unparsed message from an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum UniversalMessage {
    /// Fully parsed message.
    Parsed(ParsedMessage),
    /// Raw JSON from the agent (failed to parse).
    Unparsed { raw: serde_json::Value },
}

impl UniversalMessage {
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self::Parsed(ParsedMessage {
            role: role.into(),
            parts: vec![UniversalMessagePart::Text { text: text.into() }],
            metadata: Default::default(),
        })
    }

    pub fn full_text(&self) -> String {
        match self {
            Self::Parsed(p) => p.parts.iter().filter_map(|part| {
                if let UniversalMessagePart::Text { text } = part { Some(text.as_str()) } else { None }
            }).collect::<Vec<_>>().join(""),
            Self::Unparsed { raw } => raw.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedMessage {
    pub role: String,
    pub parts: Vec<UniversalMessagePart>,
    pub metadata: serde_json::Value,
}

/// One piece of content within a message.
/// Covers all agent output types across Claude, Codex, OpenCode, Amp, Pi, Cursor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniversalMessagePart {
    /// Text content.
    Text { text: String },
    /// Tool call initiated by the agent.
    ToolCall { id: String, name: String, input: serde_json::Value },
    /// Result of a tool call.
    ToolResult { tool_use_id: String, output: String, is_error: bool },
    /// Thinking/reasoning block (Claude extended thinking, Codex reasoning).
    Thinking { text: String, #[serde(default, skip_serializing_if = "Option::is_none")] signature: Option<String> },
    /// File content (Codex images / Cursor files).
    File { name: String, content: String, mime_type: Option<String> },
    /// Image (base64 encoded).
    Image { data: String, mime_type: String },
    /// Error inside a message.
    Error { message: String },
    /// Unknown part — preserved verbatim.
    Unknown { raw: serde_json::Value },
}

impl UniversalMessagePart {
    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text { text } = self { Some(text) } else { None }
    }
    pub fn is_tool_call(&self) -> bool { matches!(self, Self::ToolCall { .. }) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn text_helper() {
        let m = UniversalMessage::text("assistant", "hello BKG");
        assert_eq!(m.full_text(), "hello BKG");
    }
    #[test] fn tool_call_part() {
        let p = UniversalMessagePart::ToolCall { id: "t1".into(), name: "bash".into(), input: serde_json::json!({"cmd":"ls"}) };
        assert!(p.is_tool_call());
        assert!(p.as_text().is_none());
    }
}