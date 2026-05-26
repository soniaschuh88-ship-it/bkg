use std::collections::HashMap;
use crate::schema::{EventSchema, EventSchemaId, SchemaVersion};

#[derive(Debug, thiserror::Error)]
pub enum SchemaRegistryError {
    #[error("schema '{0}' not found")]
    NotFound(EventSchemaId),
    #[error("schema '{id}' already registered with version {existing}, cannot register version {attempted}")]
    AlreadyRegistered { id: EventSchemaId, existing: SchemaVersion, attempted: SchemaVersion },
    #[error("schema '{id}' version {version} is incompatible: {reason}")]
    Incompatible { id: EventSchemaId, version: SchemaVersion, reason: String },
}

/// Central registry for all event schemas in DELPHOS.
/// Single source of truth. All schemas must be registered before any event of that type is emitted.
#[derive(Debug, Default)]
pub struct EventSchemaRegistry { schemas: HashMap<EventSchemaId, EventSchema> }

impl EventSchemaRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, schema: EventSchema) -> Result<(), SchemaRegistryError> {
        if let Some(existing) = self.schemas.get(&schema.id) {
            if existing.version != schema.version && !schema.version.is_compatible_with(&existing.version) {
                return Err(SchemaRegistryError::Incompatible { id: schema.id.clone(), version: schema.version, reason: "major version mismatch".into() });
            }
        }
        self.schemas.insert(schema.id.clone(), schema);
        Ok(())
    }

    pub fn get(&self, id: &EventSchemaId) -> Result<&EventSchema, SchemaRegistryError> {
        self.schemas.get(id).ok_or_else(|| SchemaRegistryError::NotFound(id.clone()))
    }

    pub fn is_registered(&self, id: &EventSchemaId) -> bool { self.schemas.contains_key(id) }
    pub fn count(&self) -> usize { self.schemas.len() }
    pub fn all(&self) -> Vec<&EventSchema> { let mut v: Vec<_>=self.schemas.values().collect(); v.sort_by(|a,b|a.id.0.cmp(&b.id.0)); v }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;
    use crate::schema::{EventSchema, SchemaVersion};

    fn task_created() -> EventSchema {
        EventSchema::new("task.created", SchemaVersion::V1, RealmId::Telum, "A task was created")
            .with_targets(vec![RealmId::Katoptron])
    }

    #[test] fn register_and_get() {
        let mut r=EventSchemaRegistry::new();
        r.register(task_created()).unwrap();
        assert!(r.is_registered(&EventSchemaId::new("task.created")));
        assert_eq!(r.count(),1);
    }
    #[test] fn not_found() {
        let r=EventSchemaRegistry::new();
        assert!(r.get(&EventSchemaId::new("nonexistent")).is_err());
    }
    #[test] fn sorted_list() {
        let mut r=EventSchemaRegistry::new();
        r.register(EventSchema::new("z.event",SchemaVersion::V1,RealmId::Styx,"")).unwrap();
        r.register(EventSchema::new("a.event",SchemaVersion::V1,RealmId::Styx,"")).unwrap();
        let all=r.all(); assert_eq!(all[0].id.0,"a.event"); assert_eq!(all[1].id.0,"z.event");
    }
    #[test] fn schema_version_compat() { assert!(SchemaVersion::V1.is_compatible_with(&SchemaVersion::new(1,5))); assert!(!SchemaVersion::V1.is_compatible_with(&SchemaVersion::new(2,0))); }
}