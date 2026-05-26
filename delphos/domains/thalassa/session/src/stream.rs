//! SSE (Server-Sent Events) streaming helpers. Single source of truth.
//! Ported from sandbox-agent event streaming + SSE serialization.

use serde::{Deserialize, Serialize};
use crate::event::UniversalEvent;

/// An SSE-formatted event line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
    pub id: Option<String>,
}

impl SseEvent {
    pub fn from_universal(ev: &UniversalEvent) -> Self {
        Self {
            event: ev.data.event_type().to_string(),
            data: serde_json::to_string(ev).unwrap_or_else(|_| "{}".to_string()),
            id: Some(ev.id.to_string()),
        }
    }

    pub fn to_sse_string(&self) -> String {
        let mut s = String::new();
        if let Some(id) = &self.id { s.push_str(&format!("id: {id}\n")); }
        s.push_str(&format!("event: {}\n", self.event));
        for line in self.data.lines() {
            s.push_str(&format!("data: {line}\n"));
        }
        s.push('\n');
        s
    }

    pub fn ping() -> Self {
        Self { event: "ping".into(), data: r#"{"type":"ping"}"#.into(), id: None }
    }

    pub fn done() -> Self {
        Self { event: "done".into(), data: r#"{"type":"done"}"#.into(), id: None }
    }
}

/// An iterator over SSE events that can be used with Axum's SSE handler.
pub struct EventStream {
    receiver: tokio::sync::broadcast::Receiver<UniversalEvent>,
}

impl EventStream {
    pub fn new(receiver: tokio::sync::broadcast::Receiver<UniversalEvent>) -> Self {
        Self { receiver }
    }

    pub async fn next_sse(&mut self) -> Option<SseEvent> {
        match self.receiver.recv().await {
            Ok(ev) => {
                let sse = SseEvent::from_universal(&ev);
                Some(sse)
            }
            Err(_) => Some(SseEvent::done()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_agents::AgentId;
    use crate::event::UniversalEventData;
    #[test] fn sse_format() {
        let ev = UniversalEvent::new(5, "sess", AgentId::Mock, UniversalEventData::Delta { text: "hello".into(), part_index: 0 });
        let sse = SseEvent::from_universal(&ev);
        assert_eq!(sse.event, "delta");
        assert!(sse.id.as_deref() == Some("5"));
        let text = sse.to_sse_string();
        assert!(text.starts_with("id: 5\n"));
        assert!(text.contains("event: delta\n"));
    }
    #[test] fn ping_and_done() {
        assert_eq!(SseEvent::ping().event, "ping");
        assert_eq!(SseEvent::done().event, "done");
    }
}