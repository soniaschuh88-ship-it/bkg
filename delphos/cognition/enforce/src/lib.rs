//! # bkg-enforce — hard enforcement layer
//! Sealed traits, invariant guards, no-bypass types.
//! Single source of truth. One module, one location.
//!
//! ## What lives here
//! - `Sealed` trait — prevents external implementations
//! - `InvariantGuard` — compile+runtime invariant checks
//! - `NoBypass<T>` — wrapper requiring pipeline passage
//! - `WorkspaceLints` — documented lint rules (enforced via Cargo.toml)
//! - `enforce!` / `assert_invariant!` macros

pub mod guards;
pub mod lints;
pub mod no_bypass;
pub mod sealed;

pub use guards::{InvariantGuard, InvariantViolated};
pub use no_bypass::NoBypass;
pub use sealed::Sealed;
