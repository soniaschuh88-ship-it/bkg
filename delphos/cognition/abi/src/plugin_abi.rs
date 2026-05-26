// plugin_abi.rs — plugin contribution format.
use serde::{Deserialize, Serialize};
use crate::envelope::AbiEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifestPayload {
    pub plugin_id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    /// UI slot names this plugin contributes to.
    pub ui_slots: Vec<String>,
    /// Dashboard view IDs contributed.
    pub dashboard_views: Vec<String>,
    /// Runtime env vars injected by this plugin.
    pub env_vars: Vec<String>,
    /// Prompt fragment IDs contributed.
    pub prompt_contributions: Vec<String>,
}
pub type PluginManifestEnvelope = AbiEnvelope<PluginManifestPayload>;