//! bkg-plugins — plugin discovery, YAML manifest, UI slots, prompt contributions.
//! Single source of truth for all plugin management in DELPHOS.
pub mod loader; pub mod manifest; pub mod registry; pub mod slot;
pub use manifest::{PluginManifest, PluginId};
pub use registry::PluginRegistry;
pub use slot::{UiSlot, PromptContribution};
