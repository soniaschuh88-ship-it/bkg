use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum IntentKind { CreateTask, ReviewCode, DebugSystem, PlanMission, ConfigureAgent, MonitorHealth, Unknown }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct OperatorIntent {
    pub id: String, pub kind: IntentKind, pub confidence: f64,
    pub context: serde_json::Value, pub inferred_at: DateTime<Utc>,
}
impl OperatorIntent {
    pub fn new(kind: IntentKind, confidence: f64) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), kind,
               confidence: confidence.clamp(0.0, 1.0),
               context: serde_json::Value::Null, inferred_at: Utc::now() }
    }
    pub fn with_context(mut self, c: serde_json::Value) -> Self { self.context = c; self }
    pub fn is_high_confidence(&self) -> bool { self.confidence >= 0.7 }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn high_conf() { assert!(OperatorIntent::new(IntentKind::CreateTask, 0.9).is_high_confidence()); }
    #[test] fn low_conf()  { assert!(!OperatorIntent::new(IntentKind::Unknown, 0.3).is_high_confidence()); }
}
