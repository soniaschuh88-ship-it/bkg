// transition.rs — valid/invalid state transition classification.
use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

#[derive(Debug, Clone, thiserror::Error)]
pub enum TransitionError {
    #[error("invalid transition in realm {realm}: {reason}")]
    InvalidTransition { realm: RealmId, reason: String },
    #[error("causality violation: {0}")]
    CausalityViolation(String),
    #[error("replay determinism failure: expected hash {expected}, got {actual}")]
    DeterminismFailure { expected: String, actual: String },
    #[error("duplicate lamport {0} in realm — concurrent events are forbidden")]
    DuplicateLamport(u64),
}

/// A record of one state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_version: u64,
    pub to_version: u64,
    pub event_type: String,
    pub event_hash: String,
    pub valid: bool,
    pub error: Option<String>,
}

impl StateTransition {
    pub fn ok(from: u64, to: u64, event_type: &str, hash: &str) -> Self {
        Self { from_version: from, to_version: to, event_type: event_type.to_string(), event_hash: hash.to_string(), valid: true, error: None }
    }
    pub fn failed(from: u64, event_type: &str, error: &str) -> Self {
        Self { from_version: from, to_version: from, event_type: event_type.to_string(), event_hash: String::new(), valid: false, error: Some(error.to_string()) }
    }
}
