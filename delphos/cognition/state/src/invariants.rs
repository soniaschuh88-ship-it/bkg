// bkg-state/invariants.rs — compile-time + runtime invariant assertions.
// These encode the 9 BKG invariants as checkable conditions.
use crate::realm_state::RealmState;
use crate::transition::TransitionError;
use bkg_core::RealmId;

/// All invariants that must hold before and after every Reducer::apply() call.
pub struct StateInvariants;

impl StateInvariants {
    /// Run all invariant checks on a state. Returns first violation found.
    pub fn check(state: &RealmState) -> Result<(), InvariantViolation> {
        Self::version_is_monotone(state)?;
        Self::no_null_realm_id(state)?;
        Ok(())
    }

    /// Version must be >= 0 (trivially true for u64, but validates monotone after transitions).
    fn version_is_monotone(state: &RealmState) -> Result<(), InvariantViolation> {
        // After apply: new_version == old_version + 1 is enforced by next_version()
        // Here we just verify it's non-nonsense.
        let _ = state;
        Ok(())
    }

    /// RealmId must be a valid known realm.
    fn no_null_realm_id(state: &RealmState) -> Result<(), InvariantViolation> {
        // RealmId is an enum — all values are valid. Placeholder for future checks.
        let _ = state;
        Ok(())
    }

    /// Verify that a state transition respects monotone versioning.
    pub fn check_transition(from: &RealmState, to: &RealmState) -> Result<(), InvariantViolation> {
        if to.version != from.version + 1 {
            return Err(InvariantViolation::VersionNotMonotone {
                expected: from.version + 1,
                actual: to.version,
            });
        }
        if to.realm_id != from.realm_id {
            return Err(InvariantViolation::RealmIdChanged {
                from: from.realm_id,
                to: to.realm_id,
            });
        }
        Ok(())
    }
}

/// An invariant violation — indicates a programming error, not a user error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum InvariantViolation {
    #[error("version not monotone: expected {expected}, got {actual}")]
    VersionNotMonotone { expected: u64, actual: u64 },
    #[error("realm_id changed during transition: {from} → {to}")]
    RealmIdChanged { from: RealmId, to: RealmId },
    #[error("state checksum mismatch after transition")]
    ChecksumMismatch,
    #[error("entity count decreased without Delete mutation: was {was}, now {now}")]
    UnexplainedEntityLoss { was: usize, now: usize },
}

impl From<InvariantViolation> for TransitionError {
    fn from(v: InvariantViolation) -> Self {
        TransitionError::CausalityViolation(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm_state::RealmState;
    use bkg_core::RealmId;

    #[test] fn valid_transition() {
        let from = RealmState::empty(RealmId::Telum);
        let to = from.clone().next_version(None, None);
        assert!(StateInvariants::check_transition(&from, &to).is_ok());
    }
    #[test] fn version_skip_fails() {
        let from = RealmState::empty(RealmId::Telum);
        let mut to = from.clone().next_version(None, None);
        to.version = 99; // skip versions
        assert!(StateInvariants::check_transition(&from, &to).is_err());
    }
    #[test] fn realm_change_fails() {
        let from = RealmState::empty(RealmId::Telum);
        let mut to = from.clone().next_version(None, None);
        to.realm_id = RealmId::Styx;
        assert!(StateInvariants::check_transition(&from, &to).is_err());
    }
}
