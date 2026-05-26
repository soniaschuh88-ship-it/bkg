use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum PrStrategy { #[default] Squash, Merge, Rebase }
impl PrStrategy { pub fn as_str(self)->&'static str{match self{Self::Squash=>"squash",Self::Merge=>"merge",Self::Rebase=>"rebase"}} }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: Option<u64>, pub title: String, pub body: Option<String>,
    pub head_branch: String, pub base_branch: String, pub strategy: PrStrategy,
    pub url: Option<String>, pub state: String, pub created_at: DateTime<Utc>,
}
impl PullRequest {
    pub fn new(title: impl Into<String>, head: impl Into<String>, base: impl Into<String>) -> Self {
        Self{number:None,title:title.into(),body:None,head_branch:head.into(),base_branch:base.into(),strategy:PrStrategy::default(),url:None,state:"open".into(),created_at:Utc::now()}
    }
    pub fn with_strategy(mut self, s: PrStrategy) -> Self { self.strategy=s; self }
    pub fn is_open(&self) -> bool { self.state=="open" }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create_pr() { let pr=PullRequest::new("feat: add BKG","bkg/task-1","main").with_strategy(PrStrategy::Squash); assert!(pr.is_open()); assert_eq!(pr.strategy,PrStrategy::Squash); }
}
