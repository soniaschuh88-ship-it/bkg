use serde::{Deserialize, Serialize};
use crate::auth::GithubAuth;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubIssue { pub number: u64, pub title: String, pub body: Option<String>, pub labels: Vec<String>, pub state: String, pub url: String }
impl GithubIssue {
    pub fn to_task_title(&self) -> String { format!("#{}: {}", self.number, self.title) }
    pub fn is_open(&self) -> bool { self.state == "open" }
}
#[derive(Debug, Clone)]
pub struct IssueImport { pub auth: GithubAuth, pub label_filter: Vec<String>, pub state_filter: String }
impl IssueImport {
    pub fn new(auth: GithubAuth) -> Self { Self{auth,label_filter:vec![],state_filter:"open".into()} }
    pub fn with_label(mut self, l: impl Into<String>) -> Self { self.label_filter.push(l.into()); self }
    pub fn import_url(&self) -> String { format!("{}/issues?state={}", self.auth.api_base(), self.state_filter) }
}
