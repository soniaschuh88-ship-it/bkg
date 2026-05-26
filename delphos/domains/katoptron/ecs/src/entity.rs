use serde::{Deserialize, Serialize};

/// Stable numeric entity handle. Generation prevents use-after-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(pub u64);
impl EntityId { pub fn new(n: u64) -> Self { Self(n) } }
impl std::fmt::Display for EntityId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "E{}", self.0) } }

/// Generation counter — prevents stale entity references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Generation(pub u32);
impl Generation { pub fn next(self) -> Self { Self(self.0 + 1) } }

/// An entity handle: (id, generation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity { pub id: EntityId, pub generation: Generation }
impl Entity {
    pub fn new(id: u64, generation: u32) -> Self { Self { id: EntityId(id), generation: Generation(generation) } }
    pub fn is_alive(&self, current_gen: Generation) -> bool { self.generation == current_gen }
}
impl std::fmt::Display for Entity { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}g{}", self.id, self.generation.0) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn entity_display() { let e = Entity::new(1, 0); assert_eq!(e.to_string(), "E1g0"); }
    #[test] fn generation_check() { let e = Entity::new(5, 2); assert!(e.is_alive(Generation(2))); assert!(!e.is_alive(Generation(3))); }
    #[test] fn ordering() { let a = EntityId(1); let b = EntityId(2); assert!(a < b); }
}
