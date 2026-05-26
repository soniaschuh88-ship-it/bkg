//! bkg-identity — deterministic lineage and ancestry IDs.
//! Without this: timeline forking becomes uncontrollable.
//! Single source of truth for all identity derivation in DELPHOS.
pub mod id; pub mod lineage; pub mod realm_identity;
pub use id::{DeterministicId, IdentityError};
pub use lineage::{AncestryChain, LineageNode};
pub use realm_identity::RealmIdentity;
