pub struct Assertion { pub id: String, pub description: String, pub passed: bool }
pub struct Oracle { pub assertions: Vec<Assertion> }
impl Oracle {
    pub fn new() -> Self { Self { assertions: vec![] } }
    pub fn assert_entity_exists(&mut self, world: &crate::world::SimWorld, key: &str, desc: &str) {
        let ok = world.entities.contains_key(key);
        self.assertions.push(Assertion { id: uuid::Uuid::new_v4().to_string(), description: format!("{desc}: '{key}' exists"), passed: ok });
    }
    pub fn assert_event_count(&mut self, world: &crate::world::SimWorld, expected: usize, desc: &str) {
        let ok = world.event_count() == expected;
        self.assertions.push(Assertion { id: uuid::Uuid::new_v4().to_string(), description: format!("{desc}: count={expected}"), passed: ok });
    }
    pub fn all_passed(&self) -> bool { self.assertions.iter().all(|a| a.passed) }
    pub fn failed(&self) -> Vec<&Assertion> { self.assertions.iter().filter(|a| !a.passed).collect() }
}
impl Default for Oracle { fn default() -> Self { Self::new() } }
#[cfg(test)]
mod tests { use super::*; use crate::world::SimWorld;
    #[test] fn pass() { let mut w = SimWorld::new(); w.set_entity("T-1", serde_json::json!({})); let mut o = Oracle::new(); o.assert_entity_exists(&w, "T-1", "exists"); assert!(o.all_passed()); }
    #[test] fn fail() { let w = SimWorld::new(); let mut o = Oracle::new(); o.assert_entity_exists(&w, "missing", "fails"); assert!(!o.all_passed()); }
}
