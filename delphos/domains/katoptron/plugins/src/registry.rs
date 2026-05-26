use std::collections::BTreeMap;
use crate::{manifest::{PluginId, PluginManifest}, slot::{PromptContribution, UiSlot}};
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct PluginRegistry { plugins: BTreeMap<String, PluginManifest>, ui_slots: Vec<UiSlot>, prompts: Vec<PromptContribution> }
impl PluginRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, m: PluginManifest) { self.plugins.insert(m.id.0.clone(), m); }
    pub fn unregister(&mut self, id: &PluginId) { self.plugins.remove(&id.0); }
    pub fn get(&self, id: &PluginId) -> Option<&PluginManifest> { self.plugins.get(&id.0) }
    pub fn all_enabled(&self) -> Vec<&PluginManifest> { self.plugins.values().filter(|p| p.enabled).collect() }
    pub fn all_ui_slots(&self) -> Vec<String> { self.all_enabled().iter().flat_map(|p| p.ui_slots.iter().cloned()).collect() }
    pub fn all_dashboard_views(&self) -> Vec<String> { self.all_enabled().iter().flat_map(|p| p.dashboard_views.iter().cloned()).collect() }
    pub fn count(&self) -> usize { self.plugins.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PluginManifest;
    #[test] fn register_and_list() {
        let mut r=PluginRegistry::new();
        r.register(PluginManifest::new("p1","Plugin 1","0.1").with_ui_slot("kanban-card").with_dashboard_view("analytics"));
        assert_eq!(r.count(),1);
        assert_eq!(r.all_ui_slots().len(),1);
        assert_eq!(r.all_dashboard_views().len(),1);
    }
}
