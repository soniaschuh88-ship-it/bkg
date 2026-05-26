use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EvalScore(pub f64);
impl EvalScore {
    pub fn new(v: f64) -> Self { Self(v.clamp(0.0,1.0)) }
    pub fn perfect() -> Self { Self(1.0) }
    pub fn zero() -> Self { Self(0.0) }
    pub fn as_percent(&self) -> f64 { self.0 * 100.0 }
    pub fn band(&self) -> &'static str {
        if self.0 >= 0.9 { "A" } else if self.0 >= 0.75 { "B" } else if self.0 >= 0.6 { "C" } else if self.0 >= 0.4 { "D" } else { "F" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scorecard {
    pub id: String, pub task_id: String, pub version: u32,
    pub categories: Vec<(String, f64, EvalScore)>, // (name, weight, score)
    pub overall_score: EvalScore,
    pub created_at: DateTime<Utc>,
}
impl Scorecard {
    pub fn new(task_id: impl Into<String>, categories: Vec<(String, f64, f64)>) -> Self {
        let total_weight: f64 = categories.iter().map(|(_,w,_)| w).sum();
        let weighted: f64 = categories.iter().map(|(_,w,s)| w*s).sum();
        let overall = EvalScore::new(if total_weight>0.0{weighted/total_weight}else{0.0});
        let cats = categories.into_iter().map(|(n,w,s)|(n,w,EvalScore::new(s))).collect();
        Self { id: uuid::Uuid::new_v4().to_string(), task_id: task_id.into(), version: 1, categories: cats, overall_score: overall, created_at: Utc::now() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult { pub scorecard: Scorecard, pub evidence_ids: Vec<String>, pub follow_up_suggestions: Vec<String> }
impl EvalResult {
    pub fn new(sc: Scorecard) -> Self { Self { scorecard: sc, evidence_ids: vec![], follow_up_suggestions: vec![] } }
    pub fn add_suggestion(&mut self, s: impl Into<String>) { self.follow_up_suggestions.push(s.into()); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn weighted_score() {
        let sc = Scorecard::new("T-1", vec![("correctness".into(),0.5,0.9),("style".into(),0.3,0.7),("tests".into(),0.2,1.0)]);
        assert!(sc.overall_score.0 > 0.85);
    }
    #[test] fn band() { assert_eq!(EvalScore::new(0.95).band(), "A"); assert_eq!(EvalScore::new(0.3).band(), "F"); }
    #[test] fn clamp() { assert_eq!(EvalScore::new(1.5).0, 1.0); assert_eq!(EvalScore::new(-0.5).0, 0.0); }
}
