use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(pub String);
impl PluginId { pub fn new(s: impl Into<String>) -> Self { Self(s.into()) } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: PluginId, pub version: String, pub name: String, pub description: String,
    pub ui_slots: Vec<String>, pub dashboard_views: Vec<String>,
    pub env_vars: Vec<String>, pub prompt_contributions: Vec<String>,
    pub enabled: bool,
}
impl PluginManifest {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self{id:PluginId::new(id),name:name.into(),version:version.into(),description:String::new(),ui_slots:vec![],dashboard_views:vec![],env_vars:vec![],prompt_contributions:vec![],enabled:true}
    }
    pub fn with_ui_slot(mut self, slot: impl Into<String>) -> Self { self.ui_slots.push(slot.into()); self }
    pub fn with_dashboard_view(mut self, view: impl Into<String>) -> Self { self.dashboard_views.push(view.into()); self }
}
