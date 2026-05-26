use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubToken { pub token: String, pub scopes: Vec<String>, pub expires_at: Option<DateTime<Utc>> }
impl GithubToken { pub fn new(t: impl Into<String>) -> Self { Self{token:t.into(),scopes:vec![],expires_at:None} } pub fn is_valid(&self) -> bool { self.expires_at.map(|e|Utc::now()<e).unwrap_or(true) } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubAuth { pub owner: String, pub repo: String, pub token: Option<GithubToken> }
impl GithubAuth {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self { Self{owner:owner.into(),repo:repo.into(),token:None} }
    pub fn with_token(mut self, t: GithubToken) -> Self { self.token=Some(t); self }
    pub fn api_base(&self) -> String { format!("https://api.github.com/repos/{}/{}",self.owner,self.repo) }
    pub fn is_authenticated(&self) -> bool { self.token.as_ref().map(|t|t.is_valid()).unwrap_or(false) }
}
