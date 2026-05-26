pub mod detection; pub mod enhancer; pub mod fetch;
pub mod providers; pub mod registry; pub mod toggle; pub mod types;
pub use detection::{apply_free_filter, detect_pricing_exposed, is_free_model};
pub use registry::{ProviderAdapter, ProviderRegistry, ProviderSummary};
pub use toggle::{ProviderToggleState, ToggleMode};
pub use types::{CostConfig, ModelInput, ProviderModelConfig};