use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
const MAX: usize = 200;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct InteractionEvent { pub action: String, pub entity_id: Option<String>, pub timestamp: DateTime<Utc> }
impl InteractionEvent {
    pub fn new(action: impl Into<String>, entity_id: Option<String>) -> Self {
        Self { action: action.into(), entity_id, timestamp: Utc::now() }
    }
}
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct InteractionHistory { events: Vec<InteractionEvent> }
impl InteractionHistory {
    pub fn new() -> Self { Self::default() }
    pub fn record(&mut self, e: InteractionEvent) {
        self.events.push(e);
        if self.events.len() > MAX { self.events.remove(0); }
    }
    pub fn recent(&self, n: usize) -> Vec<&InteractionEvent> {
        let s = self.events.len().saturating_sub(n);
        self.events[s..].iter().collect()
    }
    pub fn count(&self) -> usize { self.events.len() }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn record_and_cap() {
        let mut h = InteractionHistory::new();
        for i in 0..210 { h.record(InteractionEvent::new(format!("a{i}"), None)); }
        assert!(h.count() <= MAX);
    }
}
