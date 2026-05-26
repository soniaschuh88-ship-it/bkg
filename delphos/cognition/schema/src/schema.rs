use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

/// Stable identifier for an event schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventSchemaId(pub String);
impl EventSchemaId { pub fn new(s: impl Into<String>) -> Self { Self(s.into()) } }
impl std::fmt::Display for EventSchemaId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }

/// Schema version (major.minor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaVersion { pub major: u16, pub minor: u16 }
impl SchemaVersion { pub const V1: Self = Self { major: 1, minor: 0 }; pub fn new(major: u16, minor: u16) -> Self { Self { major, minor } } pub fn is_compatible_with(&self, other: &Self) -> bool { self.major == other.major } }
impl std::fmt::Display for SchemaVersion { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}.{}", self.major, self.minor) } }

/// How old schema versions are handled during replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
pub enum MigrationStrategy {
    /// Replay as-is — safe only if schema didn't change.
    #[default] Passthrough,
    /// Transform payload before passing to reducer.
    Transform { transformer_id: String },
    /// Skip events with this old schema version.
    Skip,
    /// Fail replay if this version is encountered.
    Reject { reason: String },
}

/// Contract for one event type in DELPHOS.
/// Registered in EventSchemaRegistry before any event of this type is emitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchema {
    /// Stable event type identifier (e.g. "task.created").
    pub id: EventSchemaId,
    /// Current schema version.
    pub version: SchemaVersion,
    /// Which realm produces this event.
    pub producer_realm: RealmId,
    /// Human-readable description.
    pub description: String,
    /// Which realms should receive this event.
    pub projection_targets: Vec<RealmId>,
    /// Other event schema IDs that must have been processed before this one.
    pub causal_requirements: Vec<EventSchemaId>,
    /// How to handle older versions of this schema during replay.
    pub migration_strategy: MigrationStrategy,
    /// Whether this schema is deprecated (old producers may still emit it).
    pub deprecated: bool,
}

impl EventSchema {
    pub fn new(id: impl Into<String>, version: SchemaVersion, producer: RealmId, desc: impl Into<String>) -> Self {
        Self { id: EventSchemaId::new(id), version, producer_realm: producer, description: desc.into(), projection_targets: vec![], causal_requirements: vec![], migration_strategy: MigrationStrategy::default(), deprecated: false }
    }
    pub fn with_targets(mut self, targets: Vec<RealmId>) -> Self { self.projection_targets = targets; self }
    pub fn with_requirements(mut self, reqs: Vec<EventSchemaId>) -> Self { self.causal_requirements = reqs; self }
}