use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// An archetype = set of component types. Entities with the same component set share an archetype.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Archetype { pub component_types: BTreeSet<String> }
impl Archetype {
    pub fn new(types: impl IntoIterator<Item=impl Into<String>>) -> Self {
        Self { component_types: types.into_iter().map(|s| s.into()).collect() }
    }
    pub fn has(&self, t: &str) -> bool { self.component_types.contains(t) }
    pub fn with(mut self, t: impl Into<String>) -> Self { self.component_types.insert(t.into()); self }
    pub fn without(mut self, t: &str) -> Self { self.component_types.remove(t); self }
    pub fn key(&self) -> String { self.component_types.iter().cloned().collect::<Vec<_>>().join("+") }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn archetype_key() { let a=Archetype::new(["Status","Priority"]); assert!(a.key().contains("Status")); assert!(a.key().contains("Priority")); }
    #[test] fn has() { let a=Archetype::new(["X"]); assert!(a.has("X")); assert!(!a.has("Y")); }
    #[test] fn with_without() { let a=Archetype::new(["A"]).with("B").without("A"); assert!(a.has("B")); assert!(!a.has("A")); }
}
