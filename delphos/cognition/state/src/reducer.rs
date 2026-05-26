// reducer.rs — Reducer<E> trait: the ONLY canonical state mutator.
use serde::{Deserialize, Serialize};
use crate::realm_state::RealmState;
use crate::transition::TransitionError;

/// Every state mutation in DELPHOS must go through this trait.
/// Implementors must be pure functions: same inputs → same output, always.
/// No I/O, no randomness, no `SystemTime::now()`.
pub trait Reducer: Send + Sync + 'static {
    type Event: Send + Sync + 'static;
    /// Apply one event to the current state, returning the NEW state.
    /// The old state is consumed — structural sharing is the implementor's responsibility.
    fn apply(&self, state: RealmState, event: Self::Event) -> Result<RealmState, TransitionError>;
    fn reducer_id(&self) -> ReducerId;
}

/// Stable identifier for a Reducer type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReducerId(pub String);
impl ReducerId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
}
impl std::fmt::Display for ReducerId { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "reducer:{}", self.0) } }

/// A no-op reducer for testing — returns state unchanged.
pub struct IdentityReducer(pub ReducerId);
impl IdentityReducer { pub fn new(id: impl Into<String>) -> Self { Self(ReducerId::new(id)) } }
impl Reducer for IdentityReducer {
    type Event = serde_json::Value;
    fn apply(&self, state: RealmState, _event: Self::Event) -> Result<RealmState, TransitionError> { Ok(state) }
    fn reducer_id(&self) -> ReducerId { self.0.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn identity() {
        let r = IdentityReducer::new("test");
        let s = RealmState::empty(bkg_core::RealmId::Telum);
        let s2 = r.apply(s.clone(), serde_json::json!({})).unwrap();
        assert_eq!(s.version, s2.version);
    }
}
