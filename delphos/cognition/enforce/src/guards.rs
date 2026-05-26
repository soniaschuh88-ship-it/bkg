// guards.rs — InvariantGuard: compile+runtime invariant enforcement.
//
// Every BKG invariant is represented as a callable guard.
// Guards are MANDATORY — if a guard fails, the system halts.
// No silent ignoring of violations. No option to disable in production.
//
// Single source of truth for invariant enforcement.

use serde::{Deserialize, Serialize};
use bkg_core::RealmId;

/// An invariant was violated. This is always a programming error.
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
#[error("invariant violated [{invariant}]: {detail}")]
pub struct InvariantViolated {
    pub invariant: String,
    pub detail: String,
    pub realm: Option<RealmId>,
}

impl InvariantViolated {
    pub fn new(invariant: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { invariant: invariant.into(), detail: detail.into(), realm: None }
    }
    pub fn in_realm(mut self, r: RealmId) -> Self { self.realm = Some(r); self }
}

/// Runtime invariant checks. All return `Result`, never panic in prod.
/// In test builds, failures are immediately visible.
pub struct InvariantGuard;

impl InvariantGuard {
    // ── Invariant 1: No mutation without event ───────────────────────────────
    pub fn require_event_id(event_id: &str) -> Result<(), InvariantViolated> {
        if event_id.is_empty() {
            return Err(InvariantViolated::new(
                "no-mutation-without-event",
                "event_id is empty — every state change must have a causative event",
            ));
        }
        Ok(())
    }

    // ── Invariant 2: Realm isolation ─────────────────────────────────────────
    pub fn require_same_realm(expected: RealmId, got: RealmId) -> Result<(), InvariantViolated> {
        if expected != got {
            return Err(InvariantViolated::new(
                "realm-isolation",
                format!("expected realm {expected}, got {got}"),
            ).in_realm(expected));
        }
        Ok(())
    }

    // ── Invariant 3: Monotone lamport ─────────────────────────────────────────
    pub fn require_monotone_lamport(prev: u64, next: u64) -> Result<(), InvariantViolated> {
        if next <= prev {
            return Err(InvariantViolated::new(
                "monotone-lamport",
                format!("lamport must increase: prev={prev}, next={next}"),
            ));
        }
        Ok(())
    }

    // ── Invariant 4: Version monotone ────────────────────────────────────────
    pub fn require_monotone_version(prev: u64, next: u64) -> Result<(), InvariantViolated> {
        if next != prev + 1 {
            return Err(InvariantViolated::new(
                "monotone-version",
                format!("version must increment by 1: prev={prev}, next={next}"),
            ));
        }
        Ok(())
    }

    // ── Invariant 5: Non-empty entity ID ────────────────────────────────────
    pub fn require_entity_id(id: &str) -> Result<(), InvariantViolated> {
        if id.is_empty() {
            return Err(InvariantViolated::new(
                "non-empty-entity-id",
                "entity ID must not be empty",
            ));
        }
        Ok(())
    }

    // ── Invariant 6: Checksum integrity ──────────────────────────────────────
    pub fn require_matching_checksum(expected: &str, actual: &str) -> Result<(), InvariantViolated> {
        if expected != actual {
            return Err(InvariantViolated::new(
                "checksum-integrity",
                format!("checksum mismatch: expected {expected}, got {actual}"),
            ));
        }
        Ok(())
    }

    // ── Invariant 7: No null realm state ────────────────────────────────────
    pub fn require_realm_state_exists(exists: bool, realm: RealmId) -> Result<(), InvariantViolated> {
        if !exists {
            return Err(InvariantViolated::new(
                "no-null-realm-state",
                format!("realm {realm} has no state — replay required"),
            ).in_realm(realm));
        }
        Ok(())
    }
}

/// Compile-time-safe invariant assertion macro.
/// Panics with structured context in debug builds.
/// In release builds: returns Err.
#[macro_export]
macro_rules! assert_invariant {
    ($cond:expr, $invariant:literal, $detail:expr) => {{
        if !($cond) {
            let v = $crate::guards::InvariantViolated::new($invariant, $detail);
            #[cfg(debug_assertions)]
            panic!("INVARIANT VIOLATED: {v}");
            #[cfg(not(debug_assertions))]
            return Err(v.into());
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkg_core::RealmId;

    #[test] fn event_id_required() {
        assert!(InvariantGuard::require_event_id("").is_err());
        assert!(InvariantGuard::require_event_id("evt-1").is_ok());
    }
    #[test] fn realm_isolation() {
        assert!(InvariantGuard::require_same_realm(RealmId::Telum, RealmId::Telum).is_ok());
        assert!(InvariantGuard::require_same_realm(RealmId::Telum, RealmId::Styx).is_err());
    }
    #[test] fn monotone_lamport() {
        assert!(InvariantGuard::require_monotone_lamport(1, 2).is_ok());
        assert!(InvariantGuard::require_monotone_lamport(5, 5).is_err());
        assert!(InvariantGuard::require_monotone_lamport(5, 3).is_err());
    }
    #[test] fn monotone_version() {
        assert!(InvariantGuard::require_monotone_version(0, 1).is_ok());
        assert!(InvariantGuard::require_monotone_version(0, 2).is_err());
        assert!(InvariantGuard::require_monotone_version(3, 3).is_err());
    }
    #[test] fn checksum() {
        assert!(InvariantGuard::require_matching_checksum("abc", "abc").is_ok());
        assert!(InvariantGuard::require_matching_checksum("abc", "xyz").is_err());
    }
    #[test] fn violation_display() {
        let v = InvariantViolated::new("test", "detail");
        assert!(v.to_string().contains("test"));
    }
}
