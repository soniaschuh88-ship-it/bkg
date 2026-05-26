use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);
impl ProjectId { pub fn new() -> Self { Self(format!("P-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase())) } }
impl Default for ProjectId { fn default() -> Self { Self::new() } }
impl std::fmt::Display for ProjectId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project { pub id: ProjectId, pub name: String, pub path: std::path::PathBuf, pub settings: crate::settings::ProjectSettings, pub created_at: DateTime<Utc> }
impl Project {
    pub fn new(name: impl Into<String>, path: impl Into<std::path::PathBuf>) -> Self {
        Self { id: ProjectId::new(), name: name.into(), path: path.into(), settings: Default::default(), created_at: Utc::now() }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn create() { let p=Project::new("BKG","./"); assert!(p.id.0.starts_with("P-")); }
}