//! # bkg-schema — EventSchemaRegistry
//! Replay-safe event schema management with migration strategies.
//! Single source of truth for all event type contracts.

pub mod migration; pub mod registry; pub mod schema; pub mod version;
pub use registry::{EventSchemaRegistry, SchemaRegistryError};
pub use schema::{EventSchema, EventSchemaId, MigrationStrategy};
pub use schema::SchemaVersion;