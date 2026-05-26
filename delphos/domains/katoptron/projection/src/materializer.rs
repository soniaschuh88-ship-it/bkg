use crate::cache::{ProjectionCache, ProjectionEntry};
use chrono::Utc;

/// A function that builds a projection from raw entity JSON.
pub type MaterializerFn = fn(&str, &str, u64, &str, &[serde_json::Value]) -> serde_json::Value;

/// Runs a materializer function and stores the result in the cache.
pub struct Materializer {
    cache: ProjectionCache,
}

impl Materializer {
    pub fn new() -> Self { Self { cache: ProjectionCache::new() } }

    pub fn materialize(
        &mut self,
        realm_id: &str,
        projection_id: &str,
        state_version: u64,
        state_checksum: &str,
        entities: &[serde_json::Value],
        builder: MaterializerFn,
    ) -> &ProjectionEntry {
        if !self.cache.is_stale(realm_id, projection_id, state_checksum) {
            return self.cache.get(realm_id, projection_id).unwrap();
        }
        let data = builder(realm_id, projection_id, state_version, state_checksum, entities);
        let build_count = self.cache.get(realm_id, projection_id)
            .map(|e| e.build_count + 1).unwrap_or(1);
        let entry = ProjectionEntry {
            projection_id: projection_id.into(),
            realm_id: realm_id.into(),
            state_version,
            state_checksum: state_checksum.into(),
            data,
            built_at: Utc::now(),
            build_count,
        };
        self.cache.insert(entry);
        self.cache.get(realm_id, projection_id).unwrap()
    }

    pub fn cache(&self) -> &ProjectionCache { &self.cache }
}

impl Default for Materializer { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    fn kanban_builder(_realm: &str, _pid: &str, version: u64, _cksum: &str, entities: &[serde_json::Value]) -> serde_json::Value {
        serde_json::json!({"columns": entities.len(), "version": version})
    }
    #[test] fn materializes_on_first_call() {
        let mut m = Materializer::new();
        let e = m.materialize("telum","kanban",1,"abc",&[serde_json::json!({"id":"T-1"})],kanban_builder);
        assert_eq!(e.data["columns"], 1);
        assert_eq!(e.build_count, 1);
    }
    #[test] fn cache_hit_skips_rebuild() {
        let mut m = Materializer::new();
        m.materialize("telum","kanban",1,"abc",&[],kanban_builder);
        let e2 = m.materialize("telum","kanban",1,"abc",&[serde_json::json!({"extra":"ignored"})],kanban_builder);
        assert_eq!(e2.build_count, 1); // not rebuilt
    }
    #[test] fn stale_triggers_rebuild() {
        let mut m = Materializer::new();
        m.materialize("telum","kanban",1,"abc",&[],kanban_builder);
        let e2 = m.materialize("telum","kanban",2,"xyz",&[serde_json::json!({})],kanban_builder);
        assert_eq!(e2.build_count, 2); // rebuilt
        assert_eq!(e2.state_checksum, "xyz");
    }
}
