//! bkg-ecs — Deterministic sparse-archetype Entity Component System.
//! Single source of truth for all DELPHOS world entities.
//! Stable iteration order. Replay-safe allocation. No HashMap nondeterminism.
pub mod archetype; pub mod component; pub mod entity; pub mod query; pub mod system; pub mod world;
pub use entity::{Entity, EntityId, Generation};
pub use component::Component;
pub use world::World;
pub use query::Query;
