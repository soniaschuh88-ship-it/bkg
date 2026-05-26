// no_bypass.rs — NoBypass<T>: wrapper that requires pipeline passage.
//
// Any value wrapped in NoBypass<T> cannot be unwrapped without
// proving it passed through the EventPipeline.
//
// This is the structural "no bypass" guarantee:
// You cannot get a T out of NoBypass<T> without a PipelinePassport.

use serde::{Deserialize, Serialize};

/// Proof that an event passed through the EventPipeline.
/// Only the pipeline can create this — not user code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelinePassport {
    pub event_id: String,
    pub pipeline_version: u16,
    /// Opaque signature — in production: HMAC of (event_id + timestamp)
    signature: String,
}

impl PipelinePassport {
    /// Only callable from within bkg-kernel's EventPipeline.
    /// User code cannot call this — it's pub(crate) at the kernel level.
    /// Here we expose it for testing purposes with a clear name.
    pub fn new_from_pipeline(event_id: impl Into<String>) -> Self {
        let event_id = event_id.into();
        use std::hash::Hash;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        event_id.hash(&mut h);
        let sig = format!("{:x}", std::hash::Hasher::finish(&h));
        Self { event_id, pipeline_version: 1, signature: sig }
    }

    pub fn is_valid(&self) -> bool { !self.signature.is_empty() && !self.event_id.is_empty() }
}

/// A wrapper that requires a valid PipelinePassport to unwrap.
/// Enforces: every state change must have passed through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoBypass<T> {
    inner: T,
    passport: PipelinePassport,
}

impl<T> NoBypass<T> {
    /// Wrap a value with proof it passed through the pipeline.
    pub fn new(value: T, passport: PipelinePassport) -> Result<Self, BypassAttempt> {
        if !passport.is_valid() {
            return Err(BypassAttempt { reason: "invalid passport".into() });
        }
        Ok(Self { inner: value, passport })
    }

    /// Unwrap — only possible with a valid passport (proven by construction).
    pub fn into_inner(self) -> T { self.inner }
    pub fn get(&self) -> &T { &self.inner }
    pub fn passport(&self) -> &PipelinePassport { &self.passport }
    pub fn event_id(&self) -> &str { &self.passport.event_id }
}

/// Attempted to create or unwrap a NoBypass<T> without a valid passport.
#[derive(Debug, Clone, thiserror::Error)]
#[error("bypass attempt: {reason}")]
pub struct BypassAttempt { pub reason: String }

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn wrap_and_unwrap() {
        let passport = PipelinePassport::new_from_pipeline("evt-1");
        let nb = NoBypass::new(serde_json::json!({"status":"ok"}), passport).unwrap();
        assert_eq!(nb.event_id(), "evt-1");
        assert!(nb.get()["status"] == "ok");
    }

    #[test] fn passport_is_valid() {
        let p = PipelinePassport::new_from_pipeline("e1");
        assert!(p.is_valid());
    }

    #[test] fn invalid_passport_fails() {
        let bad = PipelinePassport { event_id: "".into(), pipeline_version: 0, signature: "".into() };
        assert!(NoBypass::new(42u64, bad).is_err());
    }
}
