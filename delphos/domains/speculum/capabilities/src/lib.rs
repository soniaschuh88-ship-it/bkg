//! bkg-capabilities — realm-scoped permission tokens.
//! Prevents agents from becoming allmächtige Götter.
//! Single source of truth for all capability grants.
pub mod capability; pub mod grant; pub mod scope;
pub use capability::{CapabilityId, CapabilitySet};
pub use grant::{CapabilityGrant, GrantStatus};
pub use scope::ExecutionScope;
