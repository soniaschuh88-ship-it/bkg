//! # bkg-abi — Universal Realm ABI
//! Typed serialization contracts with version negotiation.
//! Single source of truth for all inter-system communication in DELPHOS.
//!
//! Every cross-system message is wrapped in `AbiEnvelope<T>`.
//! This enables backward compatibility for mesh + plugin + snapshot version negotiation.

pub mod capsule_abi; pub mod envelope; pub mod event_abi;
pub mod mesh_abi; pub mod packet_abi; pub mod plugin_abi;
pub mod projection_abi; pub mod provider_abi; pub mod version;

pub use envelope::{AbiEnvelope, Symbol};
pub use version::{AbiCapability, AbiCompatibility, AbiVersion};