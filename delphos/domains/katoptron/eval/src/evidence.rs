use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSignal { pub name: String, pub value: serde_json::Value, pub weight: f64 }
impl EvalSignal { pub fn new(name: impl Into<String>, value: impl Into<serde_json::Value>, weight: f64) -> Self { Self{name:name.into(),value:value.into(),weight} } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalEvidence { pub id: String, pub task_id: String, pub signals: Vec<EvalSignal>, pub ai_commentary: Option<String>, pub collected_at: DateTime<Utc> }
impl EvalEvidence {
    pub fn new(task_id: impl Into<String>) -> Self { Self{id:uuid::Uuid::new_v4().to_string(),task_id:task_id.into(),signals:vec![],ai_commentary:None,collected_at:Utc::now()} }
    pub fn add_signal(&mut self, s: EvalSignal) { self.signals.push(s); }
    pub fn signal_count(&self) -> usize { self.signals.len() }
}
