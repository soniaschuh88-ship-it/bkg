use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
/// Reviewer verdict. Single source of truth for all review decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="SCREAMING_SNAKE_CASE")]
pub enum Verdict { Approve, Revise, Rethink, Unavailable }
impl Verdict { pub fn as_str(self) -> &'static str { match self { Self::Approve=>"APPROVE", Self::Revise=>"REVISE", Self::Rethink=>"RETHINK", Self::Unavailable=>"UNAVAILABLE" } } pub fn is_approve(self) -> bool { self == Self::Approve } }
impl std::fmt::Display for Verdict { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictRecord { pub verdict: Verdict, pub feedback: String, pub reviewer_model: Option<String>, pub decided_at: DateTime<Utc> }
impl VerdictRecord { pub fn new(v: Verdict, feedback: impl Into<String>, model: Option<String>) -> Self { Self { verdict: v, feedback: feedback.into(), reviewer_model: model, decided_at: Utc::now() } } }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn display() { assert_eq!(Verdict::Approve.to_string(), "APPROVE"); }
    #[test] fn is_approve() { assert!(Verdict::Approve.is_approve()); assert!(!Verdict::Revise.is_approve()); }
}