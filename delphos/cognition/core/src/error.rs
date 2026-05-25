use thiserror::Error;
use crate::RealmId;
#[derive(Debug,Error)]
pub enum BkgError {
    #[error("invalid hash: {0}")] InvalidHash(String),
    #[error("signature error: {0}")] SignatureError(String),
    #[error("hash chain broken at {event_id}: expected {expected}, got {actual}")] HashChainBroken{event_id:String,expected:String,actual:String},
    #[error("realm boundary: {from_realm} → {to_realm} requires contract")] RealmBoundaryViolation{from_realm:RealmId,to_realm:RealmId},
    #[error("no route from {from} to {to}")] NoRealmRoute{from:RealmId,to:RealmId},
    #[error("contract {contract_id} invalid: {reason}")] InvalidContract{contract_id:String,reason:String},
    #[error("event not found: {0}")] EventNotFound(String),
    #[error("ledger sealed")] LedgerSealed,
    #[error("duplicate event id: {0}")] DuplicateEventId(String),
    #[error("capsule not found: {0}")] CapsuleNotFound(String),
    #[error("capsule integrity error: {0}")] CapsuleIntegrityError(String),
    #[error("genesis mutation attempt")] GenesisMutationAttempt,
    #[error("genesis not initialised")] GenesisNotInitialised,
    #[error("replay divergence at {event_id}: {detail}")] ReplayDivergence{event_id:String,detail:String},
    #[error("policy denied: {0}")] PolicyDenied(String),
    #[error("memory node not found: {0}")] MemoryNodeNotFound(String),
    #[error("missing capability: {0}")] MissingCapability(String),
    #[error("SWD not found: {0}")] SwdNotFound(String),
    #[error("SWD integrity error: {0}")] SwdIntegrityError(String),
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
    #[error("serialisation error: {0}")] Serialisation(#[from] serde_json::Error),
    #[error("internal: {0}")] Internal(String),
}
pub type BkgResult<T> = Result<T,BkgError>;
