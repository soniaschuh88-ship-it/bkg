//! AppState — single in-memory + persisted state for the entire server.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use bkg_providers::{ProviderRegistry, ProviderToggleState, ToggleMode};
use bkg_telemetry::{ModelTracker, QuotaMonitor};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalProviderKeys {
    /// Admin-set global fallback keys: provider_id → api_key
    pub keys: HashMap<String, String>,
    pub default_model: Option<String>,
    pub free_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMode { Private, Cloud }
#[allow(clippy::derivable_impls)]
impl Default for AppMode { fn default() -> Self { Self::Private } }
impl AppMode {
    pub fn as_str(&self) -> &'static str { match self { Self::Private => "private", Self::Cloud => "cloud" } }
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub mode: AppMode,
    pub registry: ProviderRegistry,
    pub toggle: ProviderToggleState,
    pub tracker: ModelTracker,
    pub quota: QuotaMonitor,
    pub globals: GlobalProviderKeys,
    /// Self-registration rate-limit: ip → (count, reset_at)
    pub reg_rate: HashMap<String, (u32, chrono::DateTime<chrono::Utc>)>,
}

impl AppState {
    pub fn load(data_dir: &str) -> anyhow::Result<Self> {
        let data_dir = PathBuf::from(data_dir);
        std::fs::create_dir_all(&data_dir)?;

        let toggle = ProviderToggleState::load_from_file(&data_dir.join("providers-toggle.json"));
        let tracker = ModelTracker::open(data_dir.join("telemetry.json")).unwrap_or_default();
        let quota = QuotaMonitor::open(data_dir.join("quota.json")).unwrap_or_default();
        let globals: GlobalProviderKeys = data_dir.join("global-providers.json")
            .exists().then(|| std::fs::read_to_string(data_dir.join("global-providers.json")).ok())
            .flatten().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();

        let mut registry = ProviderRegistry::default_populated();
        // Apply toggle state
        for (id, mode) in &toggle.per_provider {
            registry.toggle_provider(id);
            if *mode == ToggleMode::FreeOnly { registry.toggle_provider(id); } // reset if needed
        }

        Ok(Self { data_dir, mode: AppMode::default(), registry, toggle, tracker, quota, globals, reg_rate: HashMap::new() })
    }

    pub fn save_globals(&self) -> anyhow::Result<()> {
        let path = self.data_dir.join("global-providers.json");
        let json = serde_json::to_string_pretty(&self.globals)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Resolve key for a provider using fallback chain:
    /// 1. User's own key  2. Admin global  3. Env var  4. Anonymous (Kilo, LLM7 only)
    pub fn resolve_provider_key(&self, provider_id: &str, user_keys: Option<&HashMap<String,String>>) -> Option<String> {
        if let Some(keys) = user_keys {
            if let Some(k) = keys.get(provider_id) { if !k.is_empty() { return Some(k.clone()); } }
        }
        if let Some(k) = self.globals.keys.get(provider_id) { if !k.is_empty() { return Some(k.clone()); } }
        bkg_providers::fetch::resolve_key(&format!("{}_API_KEY", provider_id.to_uppercase()), None)
    }

    pub fn data_dir(&self) -> &Path { &self.data_dir }
}