use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum SecretScope { Project(String), Global }
impl SecretScope {
    pub fn project(id: impl Into<String>) -> Self { Self::Project(id.into()) }
    pub fn global() -> Self { Self::Global }
    pub fn as_str(&self) -> String { match self { Self::Project(id)=>format!("project:{id}"), Self::Global=>"global".into() } }
}
impl std::fmt::Display for SecretScope { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.as_str()) } }
