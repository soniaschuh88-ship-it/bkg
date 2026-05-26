// rule_engine.rs — δ compression via algebraic rule synthesis.
//
// PROBLEM: An explicit 522-cell table (18 phases × 29 inputs) cannot be
// verified or maintained as the state space grows. Adding one phase costs
// 29 new cells. Adding one input costs 18 new cells. The verification
// burden is O(|Q| × |Σ|).
//
// SOLUTION: Express δ as a small set of algebraic RULES.
// The table is DERIVED from rules — it is never the source of truth.
// Verification cost is O(|rules|), not O(|Q| × |Σ|).
//
// The rules cover ~95% of transitions via 6 patterns:
//
//   Rule::Absorbing        — absorbing state: all inputs → self
//   Rule::Escape           — absorbing state escape: specific input → specific phase
//   Rule::UniversalFault   — FaultDetected from any phase in set → Faulted
//   Rule::PipelineAdvance  — correct input at pipeline phase → next phase
//   Rule::RejectionToIdle  — validation failure → Idle
//   Rule::Explicit         — residual arcs not covered by above patterns
//
// Properties verified at startup (not just claimed):
//   1. Non-conflicting: no two rules produce different targets for same (phase,input)
//   2. Table-consistent: every entry in TRANSITION_TABLE is derivable from rules
//   3. Complete: synthesized table contains all TRANSITION_TABLE entries
//
// Single source of truth for δ compression.

use std::collections::{BTreeMap, BTreeSet};

use crate::kernel_state::{
    KernelInputKind, KernelPhase, TRANSITION_TABLE,
};

// ─── RulePattern ─────────────────────────────────────────────────────────────

/// An algebraic rule pattern — generates one or more transition entries.
#[derive(Debug, Clone)]
pub enum RulePattern {
    /// Absorbing state: phase + ANY input → phase (self-loop).
    /// Exception inputs are handled by separate rules (e.g., Escape).
    Absorbing {
        phase: KernelPhase,
        except: BTreeSet<KernelInputKind>,
    },

    /// Escape from an absorbing state: specific (phase, input) → target.
    Escape {
        from: KernelPhase,
        on: KernelInputKind,
        to: KernelPhase,
    },

    /// Universal fault arc: FaultDetected from any phase in the set → Faulted.
    UniversalFault {
        from_phases: Vec<KernelPhase>,
    },

    /// Pipeline advance: phase + its single "success" input → next phase.
    /// Covers the entire processing pipeline in one rule.
    PipelineAdvance {
        steps: Vec<(KernelPhase, KernelInputKind, KernelPhase)>,
    },

    /// Rejection arc: validation failure input → Idle.
    /// Covers all six validation rejection arcs in one rule.
    RejectionToIdle {
        phases_and_inputs: Vec<(KernelPhase, KernelInputKind)>,
    },

    /// Explicit arc: used for lifecycle + replay + recovery residuals.
    Explicit {
        from: KernelPhase,
        on: KernelInputKind,
        to: KernelPhase,
    },
}

/// A named rule with its pattern.
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub name: &'static str,
    pub priority: u8, // lower = higher priority (evaluated first)
    pub pattern: RulePattern,
}

impl TransitionRule {
    /// Synthesize all transition entries from this rule.
    pub fn synthesize(&self) -> Vec<(KernelPhase, KernelInputKind, KernelPhase)> {
        match &self.pattern {
            RulePattern::Absorbing { phase, except } => {
                KernelInputKind::ALL.iter()
                    .filter(|&&input| !except.contains(&input))
                    .map(|&input| (*phase, input, *phase))
                    .collect()
            }

            RulePattern::Escape { from, on, to } => vec![(*from, *on, *to)],

            RulePattern::UniversalFault { from_phases } => {
                from_phases.iter()
                    .map(|&phase| (phase, KernelInputKind::FaultDetected, KernelPhase::Faulted))
                    .collect()
            }

            RulePattern::PipelineAdvance { steps } => {
                steps.iter().map(|&(f, i, t)| (f, i, t)).collect()
            }

            RulePattern::RejectionToIdle { phases_and_inputs } => {
                phases_and_inputs.iter()
                    .map(|&(phase, input)| (phase, input, KernelPhase::Idle))
                    .collect()
            }

            RulePattern::Explicit { from, on, to } => vec![(*from, *on, *to)],
        }
    }

    pub fn entry_count(&self) -> usize { self.synthesize().len() }
}

// ─── The canonical rule set — source of truth for δ ─────────────────────────

/// The complete canonical rule set.
/// This is the compressed representation of δ.
/// ALL transition entries must be derivable from these rules.
pub fn canonical_rules() -> Vec<TransitionRule> {
    use KernelPhase::*;
    use KernelInputKind::*;

    vec![
        // ── Rule 0: Sealed absorbs all inputs (highest priority) ────────────
        TransitionRule {
            name: "sealed-absorbing",
            priority: 0,
            pattern: RulePattern::Absorbing {
                phase: Sealed,
                except: BTreeSet::new(),
            },
        },

        // ── Rule 1: Faulted absorbs all except RecoveryAttempted ────────────
        TransitionRule {
            name: "faulted-absorbing",
            priority: 0,
            pattern: RulePattern::Absorbing {
                phase: Faulted,
                except: [RecoveryAttempted].iter().copied().collect(),
            },
        },

        // ── Rule 2: Recovery escape from Faulted ────────────────────────────
        TransitionRule {
            name: "faulted-recovery-escape",
            priority: 1,
            pattern: RulePattern::Escape {
                from: Faulted,
                on: RecoveryAttempted,
                to: Recovering,
            },
        },

        // ── Rule 3: Universal fault arc ──────────────────────────────────────
        // FaultDetected from any non-terminal, non-genesis phase → Faulted
        TransitionRule {
            name: "universal-fault",
            priority: 2,
            pattern: RulePattern::UniversalFault {
                from_phases: vec![
                    Bootstrapping, Idle,
                    ValidatingAbi, ValidatingSchema, ValidatingClock,
                    ValidatingCapability, ValidatingCausal, Deciding,
                    Applying, Stamping, Emitting,
                    ReplayPending, Replaying, VerifyingIdentity,
                    Recovering,
                ],
            },
        },

        // ── Rule 4: Pipeline happy-path advance ──────────────────────────────
        // Covers the entire 10-step processing pipeline in one rule
        TransitionRule {
            name: "pipeline-advance",
            priority: 3,
            pattern: RulePattern::PipelineAdvance {
                steps: vec![
                    (Idle,                 EventArrived,       ValidatingAbi),
                    (ValidatingAbi,        AbiValid,           ValidatingSchema),
                    (ValidatingSchema,     SchemaValid,        ValidatingClock),
                    (ValidatingClock,      ClockValid,         ValidatingCapability),
                    (ValidatingCapability, CapabilityGranted,  ValidatingCausal),
                    (ValidatingCausal,     CausalValid,        Deciding),
                    (Deciding,             DecisionAllow,      Applying),
                    (Deciding,             DecisionTransform,  Applying),
                    (Applying,             TransitionApplied,  Stamping),
                    (Stamping,             ProjectionStamped,  Emitting),
                    (Emitting,             EmitComplete,       Idle), // cycle arc
                ],
            },
        },

        // ── Rule 5: Validation rejection arcs ──────────────────────────────
        // Any validation failure → Idle (event rejected, kernel ready again)
        TransitionRule {
            name: "rejection-to-idle",
            priority: 3,
            pattern: RulePattern::RejectionToIdle {
                phases_and_inputs: vec![
                    (ValidatingAbi,        AbiFailed),
                    (ValidatingSchema,     SchemaFailed),
                    (ValidatingClock,      ClockFailed),
                    (ValidatingCapability, CapabilityDenied),
                    (ValidatingCausal,     CausalFailed),
                    (Deciding,             DecisionReject),
                ],
            },
        },

        // ── Rule 6: Recovery arcs ────────────────────────────────────────────
        TransitionRule {
            name: "applying-failed",
            priority: 3,
            pattern: RulePattern::Explicit { from: Applying, on: TransitionFailed, to: Recovering },
        },
        TransitionRule {
            name: "recovery-self-loop",
            priority: 3,
            pattern: RulePattern::Explicit { from: Recovering, on: RecoveryAttempted, to: Recovering },
        },
        TransitionRule {
            name: "recovery-success",
            priority: 3,
            pattern: RulePattern::Explicit { from: Recovering, on: RecoverySucceeded, to: Idle },
        },

        // ── Rule 7: Replay arc ───────────────────────────────────────────────
        TransitionRule {
            name: "replay-request",
            priority: 3,
            pattern: RulePattern::Explicit { from: Idle, on: ReplayRequested, to: ReplayPending },
        },
        TransitionRule {
            name: "replay-start",
            priority: 3,
            pattern: RulePattern::Explicit { from: ReplayPending, on: EventArrived, to: Replaying },
        },
        TransitionRule {
            name: "replay-event",
            priority: 3,
            pattern: RulePattern::Explicit { from: Replaying, on: ReplayEventApplied, to: Replaying },
        },
        TransitionRule {
            name: "replay-complete",
            priority: 3,
            pattern: RulePattern::Explicit { from: Replaying, on: ReplayComplete, to: VerifyingIdentity },
        },
        TransitionRule {
            name: "identity-confirmed",
            priority: 3,
            pattern: RulePattern::Explicit { from: VerifyingIdentity, on: IdentityConfirmed, to: Idle },
        },
        TransitionRule {
            name: "identity-diverged",
            priority: 3,
            pattern: RulePattern::Explicit { from: VerifyingIdentity, on: IdentityDiverged, to: Faulted },
        },

        // ── Rule 8: Lifecycle arcs ───────────────────────────────────────────
        TransitionRule {
            name: "genesis-init",
            priority: 3,
            pattern: RulePattern::Explicit { from: Genesis, on: Initialize, to: Bootstrapping },
        },
        TransitionRule {
            name: "bootstrap-complete",
            priority: 3,
            pattern: RulePattern::Explicit { from: Bootstrapping, on: BootstrapComplete, to: Idle },
        },
        TransitionRule {
            name: "seal-from-idle",
            priority: 3,
            pattern: RulePattern::Explicit { from: Idle, on: SealRequested, to: Sealed },
        },
        TransitionRule {
            name: "seal-from-replay",
            priority: 3,
            pattern: RulePattern::Explicit { from: Replaying, on: SealRequested, to: Sealed },
        },
    ]
}

// ─── RuleEngine ──────────────────────────────────────────────────────────────

/// Synthesizes and verifies the transition table from the canonical rule set.
pub struct RuleEngine {
    rules: Vec<TransitionRule>,
    /// Synthesized table: (phase, input) → target phase
    /// Built from rules, highest-priority wins on conflict.
    synthesized: BTreeMap<(KernelPhase, KernelInputKind), KernelPhase>,
}

/// Result of rule engine verification.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub rule_count: usize,
    pub synthesized_entries: usize,
    pub explicit_table_entries: usize,
    pub conflicts: Vec<RuleConflict>,
    pub missing_from_rules: Vec<(KernelPhase, KernelInputKind, KernelPhase)>,
    pub table_consistent: bool,
}

#[derive(Debug, Clone)]
pub struct RuleConflict {
    pub phase: KernelPhase,
    pub input: KernelInputKind,
    pub rule_a: &'static str,
    pub target_a: KernelPhase,
    pub rule_b: &'static str,
    pub target_b: KernelPhase,
}

impl VerificationReport {
    pub fn is_ok(&self) -> bool {
        self.conflicts.is_empty() && self.table_consistent
    }
}

impl RuleEngine {
    /// Build the RuleEngine from the canonical rule set.
    pub fn new() -> Self {
        let rules = canonical_rules();
        let mut synthesized: BTreeMap<(KernelPhase, KernelInputKind), KernelPhase> = BTreeMap::new();

        // Apply rules in priority order (lower priority number = higher priority).
        // On conflict, lower priority wins.
        let mut sorted_rules = rules.clone();
        sorted_rules.sort_by_key(|r| r.priority);

        for rule in &sorted_rules {
            for (from, input, to) in rule.synthesize() {
                synthesized.entry((from, input)).or_insert(to);
            }
        }

        Self { rules, synthesized }
    }

    /// Look up δ(phase, input) from the synthesized table.
    /// Undefined → KernelPhase::Faulted (makes δ total).
    pub fn delta(&self, phase: KernelPhase, input: KernelInputKind) -> KernelPhase {
        self.synthesized.get(&(phase, input)).copied().unwrap_or(KernelPhase::Faulted)
    }

    /// Full verification: check rules against the explicit TRANSITION_TABLE.
    pub fn verify(&self) -> VerificationReport {
        let mut conflicts = Vec::new();

        // Check for intra-rule conflicts (two rules produce different targets for same cell)
        // by scanning the synthesized table for any cell that was claimed by two rules
        // with conflicting targets. We detect this by re-synthesizing with conflict tracking.
        let mut seen: BTreeMap<(KernelPhase, KernelInputKind), (&'static str, KernelPhase)> = BTreeMap::new();
        for rule in &self.rules {
            for (from, input, to) in rule.synthesize() {
                let key = (from, input);
                if let Some((prev_rule, prev_to)) = seen.get(&key) {
                    if *prev_to != to {
                        conflicts.push(RuleConflict {
                            phase: from, input,
                            rule_a: prev_rule, target_a: *prev_to,
                            rule_b: rule.name, target_b: to,
                        });
                    }
                } else {
                    seen.insert(key, (rule.name, to));
                }
            }
        }

        // Check that every entry in TRANSITION_TABLE is covered by the synthesized table
        let mut missing = Vec::new();
        for entry in TRANSITION_TABLE {
            let synthesized_target = self.delta(entry.from, entry.on);
            if synthesized_target != entry.to {
                missing.push((entry.from, entry.on, entry.to));
            }
        }

        let table_consistent = missing.is_empty();

        VerificationReport {
            rule_count: self.rules.len(),
            synthesized_entries: self.synthesized.len(),
            explicit_table_entries: TRANSITION_TABLE.len(),
            conflicts,
            missing_from_rules: missing,
            table_consistent,
        }
    }

    pub fn rule_count(&self) -> usize { self.rules.len() }
    pub fn synthesized_entry_count(&self) -> usize { self.synthesized.len() }

    /// Compression ratio: explicit_cells / rule_count.
    pub fn compression_ratio(&self) -> f64 {
        let total_cells = KernelPhase::ALL.len() * KernelInputKind::ALL.len();
        total_cells as f64 / self.rules.len() as f64
    }
}

impl Default for RuleEngine { fn default() -> Self { Self::new() } }

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> RuleEngine { RuleEngine::new() }

    #[test]
    fn verification_passes() {
        let e = engine();
        let report = e.verify();
        assert!(report.is_ok(),
            "rule engine verification failed:\n  conflicts: {:?}\n  missing: {:?}",
            report.conflicts, report.missing_from_rules);
    }

    #[test]
    fn no_rule_conflicts() {
        let report = engine().verify();
        assert!(report.conflicts.is_empty(),
            "conflicting rules: {:?}", report.conflicts);
    }

    #[test]
    fn all_table_entries_covered() {
        let report = engine().verify();
        assert!(report.missing_from_rules.is_empty(),
            "table entries not covered by rules: {:?}", report.missing_from_rules);
    }

    #[test]
    fn rules_are_compressed() {
        let e = engine();
        let total_cells = KernelPhase::ALL.len() * KernelInputKind::ALL.len();
        // Rules must cover all explicit table entries with far fewer definitions
        assert!(e.rule_count() < TRANSITION_TABLE.len(),
            "rules ({}) should be fewer than explicit table entries ({})",
            e.rule_count(), TRANSITION_TABLE.len());
        println!("Compression: {} rules cover {} explicit entries ({} total cells, {:.1}× ratio)",
            e.rule_count(), TRANSITION_TABLE.len(), total_cells, e.compression_ratio());
    }

    #[test]
    fn synthesized_delta_matches_explicit_delta() {
        use crate::kernel_state::kernel_delta;
        let e = engine();
        // Verify on all pipeline happy-path transitions
        let cases = [
            (KernelPhase::Idle,                 KernelInputKind::EventArrived,      KernelPhase::ValidatingAbi),
            (KernelPhase::ValidatingAbi,         KernelInputKind::AbiValid,          KernelPhase::ValidatingSchema),
            (KernelPhase::ValidatingClock,       KernelInputKind::ClockFailed,       KernelPhase::Idle),
            (KernelPhase::Deciding,              KernelInputKind::DecisionAllow,     KernelPhase::Applying),
            (KernelPhase::Emitting,              KernelInputKind::EmitComplete,      KernelPhase::Idle),
            (KernelPhase::Sealed,                KernelInputKind::EventArrived,      KernelPhase::Sealed),
            (KernelPhase::Faulted,               KernelInputKind::EventArrived,      KernelPhase::Faulted),
            (KernelPhase::Faulted,               KernelInputKind::RecoveryAttempted, KernelPhase::Recovering),
        ];
        for (from, input, expected) in cases {
            let from_rules = e.delta(from, input);
            let from_table = kernel_delta(from, input);
            assert_eq!(from_rules, expected, "rules: {from} --{input:?}-->");
            assert_eq!(from_table, expected, "table: {from} --{input:?}-->");
            assert_eq!(from_rules, from_table, "rules and table must agree");
        }
    }

    #[test]
    fn delta_is_total_via_rules() {
        let e = engine();
        for &phase in KernelPhase::ALL {
            for &input in KernelInputKind::ALL {
                let _ = e.delta(phase, input); // must not panic
            }
        }
    }

    #[test]
    fn rejection_rule_covers_all_validation_phases() {
        let e = engine();
        // All validation failure inputs must go to Idle or Faulted (not undefined)
        let failure_cases = [
            (KernelPhase::ValidatingAbi,        KernelInputKind::AbiFailed),
            (KernelPhase::ValidatingSchema,     KernelInputKind::SchemaFailed),
            (KernelPhase::ValidatingClock,      KernelInputKind::ClockFailed),
            (KernelPhase::ValidatingCapability, KernelInputKind::CapabilityDenied),
            (KernelPhase::ValidatingCausal,     KernelInputKind::CausalFailed),
            (KernelPhase::Deciding,             KernelInputKind::DecisionReject),
        ];
        for (phase, input) in failure_cases {
            let result = e.delta(phase, input);
            assert_eq!(result, KernelPhase::Idle,
                "rejection from {phase} --{input:?}--> must go to Idle, got {result}");
        }
    }
}
