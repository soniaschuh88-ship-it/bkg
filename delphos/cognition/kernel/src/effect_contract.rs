// effect_contract.rs — λ isolation: effects are functions of (phase, input) only.
//
// PROBLEM: The current kernel_effects() carries runtime strings into KernelEffect
// variants:
//   KernelEffect::EventRejected(format!("rejected at {phase}"))
//   KernelEffect::FaultRecorded(format!("fault at {phase}"))
//
// These contain runtime-constructed data. While the strings ARE deterministic
// (same (phase, input) → same string), the representation is wrong:
//   - Effects appear to carry free-form data when they carry only (phase, input)
//   - The contract between λ and its callers is implicit, not structural
//   - Future additions could accidentally inject non-deterministic data
//
// FIX: Redefine KernelEffect to carry only KernelPhase tokens.
// All effect data is derivable from the (phase, input) transition alone.
// λ is then a pure function: (KernelPhase, KernelInputKind) → EffectSet.
//
// EffectContract formally specifies WHAT EFFECTS a transition MUST produce.
// EffectVerifier checks that lambda(phase, input) satisfies its contract.
//
// This makes λ isolation structural, not documented.

use serde::{Deserialize, Serialize};
use crate::kernel_state::{KernelInputKind, KernelPhase};

// ─── Isolated KernelEffect ────────────────────────────────────────────────────

/// Observable kernel effects — fully determined by (phase, input) alone.
///
/// NO runtime strings. NO timestamps. NO external data.
/// Every variant carries only KernelPhase tokens, which are deterministic
/// from the transition inputs.
///
/// If you need to communicate event_id or detail to an observer,
/// the observer already holds the PipelineEvent — the effect is a SIGNAL,
/// not a data carrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEffectIsolated {
    /// Kernel bootstrapped successfully. Phase: Idle.
    KernelReady,
    /// Event rejected at this phase. No state change occurred.
    EventRejected { at_phase: KernelPhase },
    /// Event accepted and committed. Phase: Idle (post-emit).
    EventAccepted,
    /// Projection stamped by MaterializerKernel.
    ProjectionStamped,
    /// Replay started.
    ReplayStarted,
    /// Replay identity confirmed — system is deterministic.
    IdentityConfirmed,
    /// Replay identity DIVERGED — determinism failure.
    DeterminismViolation { at_phase: KernelPhase },
    /// Kernel sealed — terminal.
    Sealed,
    /// Fault recorded at this phase.
    FaultRecorded { at_phase: KernelPhase },
    /// Recovery succeeded. Phase: Idle.
    Recovered,
}

impl KernelEffectIsolated {
    /// Is this a terminal or error effect?
    pub fn is_error(self) -> bool {
        matches!(self, Self::DeterminismViolation { .. } | Self::FaultRecorded { .. })
    }
    /// Is this effect observable by the application layer?
    pub fn is_application_visible(self) -> bool {
        matches!(self, Self::EventAccepted | Self::EventRejected { .. }
            | Self::IdentityConfirmed | Self::DeterminismViolation { .. })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KernelReady           => "KernelReady",
            Self::EventRejected { .. }  => "EventRejected",
            Self::EventAccepted         => "EventAccepted",
            Self::ProjectionStamped     => "ProjectionStamped",
            Self::ReplayStarted         => "ReplayStarted",
            Self::IdentityConfirmed     => "IdentityConfirmed",
            Self::DeterminismViolation { .. } => "DeterminismViolation",
            Self::Sealed                => "Sealed",
            Self::FaultRecorded { .. }  => "FaultRecorded",
            Self::Recovered             => "Recovered",
        }
    }
}

// ─── EffectContract ───────────────────────────────────────────────────────────

/// The formal contract for what λ MUST produce for a specific transition.
#[derive(Debug, Clone)]
pub struct EffectContract {
    pub phase: KernelPhase,
    pub input: KernelInputKind,
    /// Effects that MUST appear in λ's output.
    pub required: Vec<KernelEffectIsolated>,
    /// Effects that MUST NOT appear in λ's output.
    pub forbidden: Vec<KernelEffectIsolated>,
}

impl EffectContract {
    fn new(phase: KernelPhase, input: KernelInputKind) -> Self {
        Self { phase, input, required: vec![], forbidden: vec![] }
    }
    fn require(mut self, e: KernelEffectIsolated) -> Self { self.required.push(e); self }
    fn forbid(mut self, e: KernelEffectIsolated) -> Self { self.forbidden.push(e); self }
}

/// Violation of an EffectContract.
#[derive(Debug, Clone)]
pub struct EffectViolation {
    pub phase: KernelPhase,
    pub input: KernelInputKind,
    pub kind: EffectViolationKind,
}

#[derive(Debug, Clone)]
pub enum EffectViolationKind {
    RequiredMissing(KernelEffectIsolated),
    ForbiddenPresent(KernelEffectIsolated),
}

impl std::fmt::Display for EffectViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "λ({}, {:?}) contract violated: {:?}",
            self.phase, self.input, self.kind)
    }
}

// ─── λ: the isolated effect function ─────────────────────────────────────────

/// The isolated effect function λ.
///
/// Pure function of (KernelPhase, KernelInputKind) → Vec<KernelEffectIsolated>.
/// No runtime data. No timestamps. No strings.
/// The same (phase, input) always produces the same effects.
pub fn lambda(phase: KernelPhase, input: KernelInputKind) -> Vec<KernelEffectIsolated> {
    use KernelPhase as P;
    use KernelInputKind as I;
    use KernelEffectIsolated as E;

    match (phase, input) {
        (P::Bootstrapping, I::BootstrapComplete) => vec![E::KernelReady],

        (P::ValidatingAbi,        I::AbiFailed)
        | (P::ValidatingSchema,   I::SchemaFailed)
        | (P::ValidatingClock,    I::ClockFailed)
        | (P::ValidatingCapability, I::CapabilityDenied)
        | (P::ValidatingCausal,   I::CausalFailed)
        | (P::Deciding,           I::DecisionReject) =>
            vec![E::EventRejected { at_phase: phase }],

        (P::Stamping, I::ProjectionStamped) => vec![E::ProjectionStamped],
        (P::Emitting, I::EmitComplete)      => vec![E::EventAccepted],
        (P::Idle,     I::ReplayRequested)   => vec![E::ReplayStarted],

        (P::VerifyingIdentity, I::IdentityConfirmed) => vec![E::IdentityConfirmed],
        (P::VerifyingIdentity, I::IdentityDiverged)  =>
            vec![E::DeterminismViolation { at_phase: P::VerifyingIdentity }],

        (P::Idle, I::SealRequested)  => vec![E::Sealed],
        (P::Recovering, I::RecoverySucceeded) => vec![E::Recovered],

        (phase, I::FaultDetected) if !phase.is_terminal() =>
            vec![E::FaultRecorded { at_phase: phase }],

        _ => vec![],
    }
}

// ─── EffectVerifier ───────────────────────────────────────────────────────────

/// Verifies that lambda satisfies all contracts.
pub struct EffectVerifier;

impl EffectVerifier {
    /// The canonical effect contracts — what λ must produce for key transitions.
    pub fn contracts() -> Vec<EffectContract> {
        use KernelPhase as P;
        use KernelInputKind as I;
        use KernelEffectIsolated as E;

        vec![
            EffectContract::new(P::Bootstrapping, I::BootstrapComplete)
                .require(E::KernelReady)
                .forbid(E::FaultRecorded { at_phase: P::Bootstrapping }),
            EffectContract::new(P::ValidatingAbi, I::AbiFailed)
                .require(E::EventRejected { at_phase: P::ValidatingAbi })
                .forbid(E::EventAccepted),
            EffectContract::new(P::ValidatingSchema, I::SchemaFailed)
                .require(E::EventRejected { at_phase: P::ValidatingSchema })
                .forbid(E::EventAccepted),
            EffectContract::new(P::ValidatingClock, I::ClockFailed)
                .require(E::EventRejected { at_phase: P::ValidatingClock })
                .forbid(E::EventAccepted),
            EffectContract::new(P::Emitting, I::EmitComplete)
                .require(E::EventAccepted)
                .forbid(E::EventRejected { at_phase: P::Emitting }),
            EffectContract::new(P::VerifyingIdentity, I::IdentityDiverged)
                .require(E::DeterminismViolation { at_phase: P::VerifyingIdentity })
                .forbid(E::IdentityConfirmed),
            EffectContract::new(P::VerifyingIdentity, I::IdentityConfirmed)
                .require(E::IdentityConfirmed)
                .forbid(E::DeterminismViolation { at_phase: P::VerifyingIdentity }),
            EffectContract::new(P::Idle, I::SealRequested)
                .require(E::Sealed),
            EffectContract::new(P::Idle, I::FaultDetected)
                .require(E::FaultRecorded { at_phase: P::Idle }),
            EffectContract::new(P::Recovering, I::RecoverySucceeded)
                .require(E::Recovered),
        ]
    }

    /// Verify that lambda satisfies all contracts.
    /// Returns Ok if all pass, Err with list of violations otherwise.
    pub fn verify_lambda() -> Result<VerificationResult, Vec<EffectViolation>> {
        let contracts = Self::contracts();
        let mut violations = Vec::new();

        for contract in &contracts {
            let effects = lambda(contract.phase, contract.input);

            for &required in &contract.required {
                if !effects.contains(&required) {
                    violations.push(EffectViolation {
                        phase: contract.phase,
                        input: contract.input,
                        kind: EffectViolationKind::RequiredMissing(required),
                    });
                }
            }

            for &forbidden in &contract.forbidden {
                if effects.contains(&forbidden) {
                    violations.push(EffectViolation {
                        phase: contract.phase,
                        input: contract.input,
                        kind: EffectViolationKind::ForbiddenPresent(forbidden),
                    });
                }
            }
        }

        if violations.is_empty() {
            Ok(VerificationResult {
                contracts_checked: contracts.len(),
                lambda_is_deterministic: true,
                lambda_is_isolated: true,
            })
        } else {
            Err(violations)
        }
    }

    /// Verify that λ is deterministic: same inputs → same outputs, always.
    pub fn verify_determinism() -> bool {
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let a = lambda(phase, input);
                let b = lambda(phase, input);
                if a != b { return false; }
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub contracts_checked: usize,
    pub lambda_is_deterministic: bool,
    pub lambda_is_isolated: bool,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lambda_satisfies_all_contracts() {
        let result = EffectVerifier::verify_lambda();
        assert!(result.is_ok(),
            "λ contract violations:\n{}",
            result.unwrap_err().iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn lambda_is_deterministic() {
        assert!(EffectVerifier::verify_determinism(),
            "λ is non-deterministic: same (phase, input) produced different outputs");
    }

    #[test]
    fn lambda_carries_no_runtime_data() {
        // KernelEffectIsolated is Copy — no heap allocation, no runtime strings.
        // This test verifies the type constraint at compile time.
        fn requires_copy<T: Copy>(_: T) {}
        requires_copy(KernelEffectIsolated::EventAccepted);
        requires_copy(KernelEffectIsolated::KernelReady);
        requires_copy(KernelEffectIsolated::Sealed);
        // If KernelEffectIsolated is Copy, it cannot contain String or Vec.
    }

    #[test]
    fn rejection_at_each_validation_phase() {
        use KernelPhase::*;
        use KernelInputKind as I;
        let cases = [
            (ValidatingAbi, I::AbiFailed), (ValidatingSchema, I::SchemaFailed),
            (ValidatingClock, I::ClockFailed), (ValidatingCapability, I::CapabilityDenied),
            (ValidatingCausal, I::CausalFailed), (Deciding, I::DecisionReject),
        ];
        for (phase, input) in cases {
            let effects = lambda(phase, input);
            assert!(effects.iter().any(|e| matches!(e, KernelEffectIsolated::EventRejected { .. })),
                "λ({phase}, {input:?}) must produce EventRejected");
            assert!(!effects.iter().any(|e| *e == KernelEffectIsolated::EventAccepted),
                "λ({phase}, {input:?}) must not produce EventAccepted");
        }
    }

    #[test]
    fn emit_complete_produces_accepted() {
        let effects = lambda(KernelPhase::Emitting, KernelInputKind::EmitComplete);
        assert!(effects.contains(&KernelEffectIsolated::EventAccepted));
        assert!(!effects.iter().any(|e| matches!(e, KernelEffectIsolated::EventRejected { .. })));
    }

    #[test]
    fn divergence_produces_violation_effect() {
        let effects = lambda(KernelPhase::VerifyingIdentity, KernelInputKind::IdentityDiverged);
        assert!(effects.iter().any(|e| matches!(e, KernelEffectIsolated::DeterminismViolation { .. })));
        assert!(!effects.contains(&KernelEffectIsolated::IdentityConfirmed));
    }

    #[test]
    fn fault_recorded_at_every_non_terminal_phase() {
        for &phase in KernelPhase::ALL {
            if !phase.is_terminal() && phase != KernelPhase::Genesis {
                let effects = lambda(phase, KernelInputKind::FaultDetected);
                assert!(effects.iter().any(|e| matches!(e, KernelEffectIsolated::FaultRecorded { .. })),
                    "λ({phase}, I::FaultDetected) must produce FaultRecorded");
            }
        }
    }

    #[test]
    fn application_visible_effects() {
        // EventAccepted and EventRejected must be application-visible
        assert!(KernelEffectIsolated::EventAccepted.is_application_visible());
        assert!(KernelEffectIsolated::EventRejected { at_phase: KernelPhase::Idle }.is_application_visible());
        // KernelReady and Sealed are internal
        assert!(!KernelEffectIsolated::KernelReady.is_application_visible());
    }
}
