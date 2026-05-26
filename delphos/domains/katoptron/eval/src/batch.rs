use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::scorecard::EvalResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalBatch {
    pub id: String, pub task_ids: Vec<String>, pub results: Vec<EvalResult>,
    pub scheduled_at: DateTime<Utc>, pub completed_at: Option<DateTime<Utc>>,
}
impl EvalBatch {
    pub fn new(task_ids: Vec<String>) -> Self { Self{id:uuid::Uuid::new_v4().to_string(),task_ids,results:vec![],scheduled_at:Utc::now(),completed_at:None} }
    pub fn add_result(&mut self, r: EvalResult) { self.results.push(r); }
    pub fn complete(&mut self) { self.completed_at = Some(Utc::now()); }
    pub fn avg_score(&self) -> f64 {
        if self.results.is_empty() { return 0.0; }
        self.results.iter().map(|r| r.scorecard.overall_score.0).sum::<f64>() / self.results.len() as f64
    }
}
