use serde::{Deserialize, Serialize};
/// 5 independent model lanes, matching Fusion's dual-scope hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelLanes {
    pub executor: Option<String>,
    pub planning: Option<String>,
    pub validator: Option<String>,
    pub title_summarizer: Option<String>,
    pub workflow_refinement: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings { pub model_lanes: ModelLanes, pub max_concurrent_tasks: Option<u32>, pub free_providers_only: bool, pub custom: serde_json::Value }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings { pub model_lanes: ModelLanes, pub default_project_id: Option<String>, pub free_providers_only: bool, pub custom: serde_json::Value }