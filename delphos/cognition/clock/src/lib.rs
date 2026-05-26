//! # bkg-clock — Realm Clock
//! Deterministic vector clocks and causal ordering.
//! **No `SystemTime::now()` anywhere in this crate.**
//! Single source of truth for all time in DELPHOS.

pub mod causal;
pub mod clock;
pub mod epoch;
pub mod tick;
pub mod timeline;

pub use causal::{CausalTime, VectorClock};
pub use clock::{RealmClock, ClockError};
pub use epoch::Epoch;
pub use tick::SequencedInstant;
pub use timeline::Timeline;
