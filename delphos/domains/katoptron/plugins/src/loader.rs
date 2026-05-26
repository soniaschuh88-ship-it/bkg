//! loader.rs — dynamic plugin module loading.
//! In production: loads .so/.wasm plugins. For now: stub with manifest validation.
use crate::manifest::PluginManifest;
use bkg_core::BkgResult;

pub struct PluginLoader;
impl PluginLoader {
    pub fn new() -> Self { Self }
    /// Validate a manifest before registering.
    pub fn validate(&self, manifest: &PluginManifest) -> BkgResult<()> {
        if manifest.id.0.is_empty() { return Err(bkg_core::BkgError::Internal("plugin id required".into())); }
        if manifest.name.is_empty() { return Err(bkg_core::BkgError::Internal("plugin name required".into())); }
        Ok(())
    }
}
impl Default for PluginLoader { fn default() -> Self { Self::new() } }
#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PluginManifest;
    #[test] fn valid_manifest() { assert!(PluginLoader::new().validate(&PluginManifest::new("p","P","1")).is_ok()); }
    #[test] fn empty_id_fails() {
        let m = PluginManifest::new("","P","1");
        assert!(PluginLoader::new().validate(&m).is_err());
    }
}
