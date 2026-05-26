use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use chrono::{DateTime, Utc};
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct AttentionMap { scores: BTreeMap<String, f64>, last_updated: Option<DateTime<Utc>> }
impl AttentionMap {
    pub fn new() -> Self { Self::default() }
    pub fn focus(&mut self, id: impl Into<String>, w: f64) {
        *self.scores.entry(id.into()).or_insert(0.0) += w;
        self.last_updated = Some(Utc::now());
    }
    pub fn score(&self, id: &str) -> f64 { self.scores.get(id).copied().unwrap_or(0.0) }
    pub fn top_n(&self, n: usize) -> Vec<(String, f64)> {
        let mut v: Vec<_> = self.scores.iter().map(|(k, &v)| (k.clone(), v)).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(n); v
    }
    pub fn decay(&mut self, factor: f64) { for v in self.scores.values_mut() { *v *= factor; } }
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn focus_top() {
        let mut a = AttentionMap::new();
        a.focus("T-1", 1.0); a.focus("T-2", 0.5); a.focus("T-1", 0.5);
        assert_eq!(a.top_n(1)[0].0, "T-1");
    }
    #[test] fn decay() {
        let mut a = AttentionMap::new();
        a.focus("X", 1.0); a.decay(0.5);
        assert!((a.score("X") - 0.5).abs() < 0.001);
    }
}
