//! bkg-projection — ProjectionCache and materializer.
//! UI reads ONLY from projections. Never the ledger directly.
//! Projections are disposable: if stale, rebuild from ledger via Reducer.
//! Single source of truth.
pub mod cache; pub mod index; pub mod materializer; pub mod subscription;
pub use cache::{ProjectionCache, ProjectionEntry};
pub use materializer::{MaterializerFn, Materializer};
pub use subscription::ProjectionSubscriber;
