//! Per-user config stored at ~/.bkg/users/<key_id>.json
use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub key_id: String,
    pub onboarded: bool,
    /// provider_id → api_key
    pub provider_keys: HashMap<String, String>,
}

impl UserConfig {
    pub fn load(data_dir: &Path, key_id: &str) -> Self {
        let path = data_dir.join("users").join(format!("{key_id}.json"));
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&s) { return cfg; }
            }
        }
        Self { key_id: key_id.to_string(), ..Default::default() }
    }

    pub fn save(&self, data_dir: &Path) -> anyhow::Result<()> {
        let dir = data_dir.join("users");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", self.key_id));
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn create(data_dir: &Path) -> Self {
        let key_id = format!("bkg_{}", crate::random_hex(24));
        let cfg = Self { key_id, ..Default::default() };
        let _ = cfg.save(data_dir);
        cfg
    }
}