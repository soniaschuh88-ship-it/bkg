//! bkg-secrets — AES-256-GCM encrypted secrets store.
//! Single source of truth. OS keychain + ~/.bkg/master.key fallback.
pub mod policy; pub mod scope; pub mod secret; pub mod store;
pub use policy::AccessPolicy;
pub use scope::SecretScope;
pub use secret::{Secret, SecretId};
pub use store::SecretsStore;
