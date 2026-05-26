use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSlot { pub slot_id: String, pub plugin_id: String, pub component_url: String }
impl UiSlot { pub fn new(slot: impl Into<String>, plugin: impl Into<String>, url: impl Into<String>) -> Self { Self{slot_id:slot.into(),plugin_id:plugin.into(),component_url:url.into()} } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContribution { pub surface: String, pub plugin_id: String, pub fragment: String, pub priority: u32 }
impl PromptContribution { pub fn new(surface: impl Into<String>, plugin: impl Into<String>, frag: impl Into<String>) -> Self { Self{surface:surface.into(),plugin_id:plugin.into(),fragment:frag.into(),priority:5} } }
