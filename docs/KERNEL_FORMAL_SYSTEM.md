# BKG — Kernel Formal System (L0–L12)

> **Single source of truth. One module, one location.**

The `bkg-kernel` crate contains a 12-layer formally verified semantic state machine.
This document specifies each layer precisely.

---

## The Formal Object

```
M = (Q, Σ, Λ, δ, λ, q₀)  — a Mealy machine

  Q  = KernelPhase         18 states, exhaustively enumerated
  Σ  = KernelInputKind     29 inputs, exhaustively enumerated
  Λ  = KernelEffect        observable side-effects (Copy type, no runtime strings)
  δ  = kernel_delta        Q × Σ → Q, TOTAL (every cell defined)
  λ  = kernel_effects      Q × Σ → Vec<KernelEffectIsolated>
  q₀ = KernelPhase::Genesis
```

Properties enforced structurally (not documented):

| Property | Enforcement |
|---|---|
| TOTAL | δ(q, σ) defined for all 522 cells; undefined → Faulted |
| DETERMINISTIC | same (q, σ) → same q' always |
| ABSORBING | Sealed absorbs all; Faulted absorbs all except RecoveryAttempted |
| MONOTONE | processing phases form a strict DAG (no backward arcs) |

---

## State Space

### KernelPhase (Q — 18 states)

```
Genesis → Bootstrapping → Idle
Idle → ValidatingAbi → ValidatingSchema → ValidatingClock
     → ValidatingCapability → ValidatingCausal → Deciding
     → Applying → Stamping → Emitting → Idle   (cycle arc)
Idle → ReplayPending → Replaying → VerifyingIdentity → Idle
Idle → Sealed                                          (terminal)
Any → Faulted                                          (terminal, via FaultDetected)
Faulted → Recovering → Idle                            (recovery arc)
```

### KernelInputKind (Σ — 29 inputs)

Grouped by category:

| Category | Inputs |
|---|---|
| Lifecycle | Initialize, BootstrapComplete |
| Event processing | EventArrived, AbiValid/Failed, SchemaValid/Failed, ClockValid/Failed, CapabilityGranted/Denied, CausalValid/Failed, DecisionAllow/Reject/Transform, TransitionApplied/Failed, ProjectionStamped, EmitComplete |
| Replay | ReplayRequested, ReplayEventApplied, ReplayComplete, IdentityConfirmed, IdentityDiverged |
| Control | SealRequested, FaultDetected, RecoveryAttempted, RecoverySucceeded |

---

## L0 — ConstraintAlgebra

**File**: `kernel_state.rs`, `constraint_algebra.rs`  
**Purpose**: δ compressed from O(|Q|×|Σ|) to O(|rules|)

Rules are the source of truth. The 522-cell table is a DERIVED artifact.

```rust
// ConstraintExpr — symbolic predicate over Q×Σ
PhaseEq(p) | PhaseIn([p]) | PhaseIsProcessing | PhaseIsTerminal
InputEq(σ) | InputIn([σ])
And(a, b) | Or(a, b) | Not(e) | True | False

// Key operations
eval(phase, input) → bool          // domain: all 522 cells
extension() → BTreeSet             // all cells where predicate holds
subsumes(other) → bool             // self.ext ⊇ other.ext
```

20 ConstraintRules → synthesize the complete table.  
Key rules:
- `sealed-absorbs-all`: `PhaseEq(Sealed)` → Self
- `faulted-absorbs-non-recovery`: `PhaseEq(Faulted) ∧ ¬InputEq(Recovery)` → Self
- `universal-fault`: `¬PhaseIsTerminal ∧ InputEq(FaultDetected)` → Faulted
- `pipeline-advance`: union of 11 exact (phase, input) pairs → NextInPipeline
- `validation-rejection`: union of 6 exact (phase, failure) pairs → Idle

**Verified properties**: non-conflicting, table-consistent, pipeline acyclic.

---

## L1 — KernelMachine

**File**: `kernel_machine.rs`  
**Purpose**: runner for the formal M=(Q,Σ,Λ,δ,λ,q₀)

```rust
pub struct KernelMachine {
    pub phase: KernelPhase,
    pub context: KernelContext,
    history: Vec<TransitionRecord>,  // append-only
    pub faults: Vec<KernelFault>,
}

// THE ONLY mutation path
pub fn step(&mut self, input: KernelInputKind) -> (KernelPhase, Vec<KernelEffect>)
```

`history` is append-only. Sequence numbers are monotone.  
Invalid transitions → `Faulted` + `KernelFault` recorded automatically.

---

## L2 — ProofCertificate

**File**: `proof_certificate.rs`  
**Purpose**: proof-carrying automaton — every transition produces a verifiable witness

```
TransitionCertificate:
  claim:         δ(from_phase, input) = to_phase
  justification: RuleMatchProof { rule_name, rule_index, produces }

ExecutionTrace:
  certificates: Vec<TransitionCertificate>
  Invariant: trace[i].to_phase == trace[i+1].from_phase

ProofChecker (TRUSTED CORE — ~30 lines):
  verify_trace(trace) → ProofCheckResult::Valid | Invalid
  Only uses canonical_rules(). No KernelMachine dependency.
```

**Trusted Computing Base**: `ProofChecker` is ~30 lines. If you trust `canonical_rules()`,
you can verify any execution without running the kernel.

---

## L3 — TraceSynthesizer

**File**: `trace_synthesizer.rs`  
**Purpose**: induce ConstraintRules from observed execution traces

**Algorithm** (Occam's Razor — broadest safe rule wins):
1. Observe (from, input, to) triples from `ExecutionTrace`
2. Group by target phase
3. Climb generalization ladder: Exact → InputGroup → PhaseGroup → CrossProduct → StructuralPattern
4. Safety check: candidate rule verified against `kernel_delta` before acceptance
5. Compare synthesized vs canonical → detect novel/uncovered/disagreeing transitions

**Outputs**: `SynthesisResult` with `table_coverage`, `novel_transitions`, `uncovered_behaviors`, `algebra_disagreements`

---

## L4 — DriftDetector

**File**: `specification_drift.rs`  
**Purpose**: detect divergence between algebra, kernel, and traces

Three pairwise checks:
- `check_algebra_vs_kernel()` — O(522 cells), must be 100% aligned
- `check_traces_vs_kernel()` — per-certificate verification
- `find_untraced_entries()` — coverage gap detection

**DriftSeverity**: `Clean | Informational | Warning | Critical`

`DriftMonitor::quick_critical_check()` runs at any time with no traces required.  
`canonical_algebra_agrees_with_kernel_everywhere` must always pass.

---

## L5 — SpecificationEntropy

**File**: `specification_entropy.rs`  
**Purpose**: measure expressiveness; detect spec collapse and over-generalization

Four metrics → composite score E(R) ∈ [0,1]:

| Metric | Weight | Detects |
|---|---|---|
| Shannon H(R) | 30% | Coverage monopoly (one rule covers everything) |
| Structural diversity D(R) | 30% | Exact-pair regression (no structural abstraction) |
| Compression ratio C(R) | 20% | Degeneration to 522 one-cell rules |
| Table coverage | 20% | Spec doesn't explain real behavior |

`collapse_risk()` = `gini × 0.6 + (1 - structural_diversity) × 0.4`

**Canonical spec**: E = 0.4+, structural_diversity > 0%, compression > 1.5×, collapse_risk < 0.8

---

## L6 — AlgebraStability

**File**: `algebra_stability.rs`  
**Purpose**: meta-invariants (rules about rules) + synthesis bounds

### HARD invariants (reject synthesis on violation)
- `kernel-alignment` — algebra must agree with kernel on all 522 cells
- `pipeline-acyclic` — processing phases must be a DAG
- `no-conflicts` — no two rules generate different targets for same cell

### SOFT invariants (warning only)
- `minimum-coverage` — explain ≥50% of TRANSITION_TABLE entries
- `expression-floor` — expressiveness above development() EntropyFloor

### PinnedRuleSet
Five rules are anchored and synthesis CANNOT remove them:
`sealed-absorbs-all`, `faulted-absorbs-non-recovery`, `universal-fault`,
`pipeline-advance`, `validation-rejection`

### SynthesisCycleGuard
Enforces across cycles: coverage monotone, expressiveness ±5%, pinned rules present.

---

## L7 — SemanticWeightLayer

**File**: `semantic_weight.rs`  
**Purpose**: measure semantic relevance — does this rule MATTER?

Three orthogonal scores → composite weight W(R) ∈ [0,1]:

| Score | Weight | Measure |
|---|---|---|
| Behavioral necessity | 35% | removing rule changes a TRANSITION_TABLE output? |
| Causal importance | 35% | rule is active for cells observed in traces? |
| Structural significance | 20% | rule encodes domain knowledge vs exact pair? |
| Reachability | 10% | rule's cells appear in traces? |

**NecessityClass**:
- `Critical` — necessary for ≥1 table entry (removing changes behavior)
- `Observational` — documents default or non-table behavior (architectural intent)
- `Redundant` — another explicit rule already covers same cells

**Anti-gaming invariant**: `sum of all causal importances = 1.0` exactly.

---

## L8 — RuleSimplifier

**File**: `rule_simplifier.rs`  
**Purpose**: safe transformation using semantic weights

Operations:
- `Remove`: rules classified Redundant (subsumed by earlier explicit rules)
- `Merge`: two rules with identical `ConstraintTarget::Phase(p)` fused by ∨-guard
- `Generalize`: exact-pair clusters → PhaseIn/InputIn rules

Every transformation checked via:
1. `passes_invariants()`: all HARD invariants pass on result
2. `passes_entropy()`: result above development() EntropyFloor

**Note**: `Self_` (absorbing self-loop) and `Phase(Faulted)` are NOT mergeable —
different semantic roles even if same output.

---

## L9 — CounterfactualAnalyzer

**File**: `counterfactual.rs`  
**Purpose**: "what would need to happen for rule R to matter?"

### StateReachabilityGraph
BFS over TRANSITION_TABLE from Genesis.
```
distances[Genesis] = 0
distances[q] = min_{(p,σ)→q} (distances[p] + 1)
```

Empirical distances:
- Genesis → 0, Bootstrapping → 1, Idle → 2
- ValidatingAbi → 3, Faulted → 4, Sealed → 3
- All 18 phases reachable, max_distance = 11

### CounterfactualWitness
Shortest valid path from Genesis to activate one cell in a rule's domain.
`makes_rule_critical = true` iff cell ∈ TRANSITION_TABLE.

### CounterfactualAnalysis
- `can_become_critical`: table entry in active domain is reachable
- `min_critical_distance`: shortest path to make rule Critical
- `fixation_risk()` = `e^(-dist/threshold)` — high near Genesis, low far away

---

## L10 — SemanticFixationGuard

**File**: `counterfactual.rs`  
**Purpose**: prevent removing rules that are easily reachable but unobserved

`near_threshold = 8` (covers complete happy path + error paths)

```
SafeToSimplify ONLY IF:
  (a) is_reachable = false         — dead code
  (b) OR all of:
      - can_become_critical = false
      - min_activation_distance > near_threshold
      - necessity != Critical
      - causal_importance < 0.01
```

**Result**: all 20 canonical rules get `Preserve` verdict.
13/20 are latently important (within 8 execution steps).

---

## L11 — CounterfactualCompetitionLayer

**File**: `counterfactual_competition.rs`  
**Purpose**: break infinite preservation bias via interference reasoning

**Unique Critical Coverage**: the decisive flag.
A rule R has unique critical coverage for cell (phase, input) iff:
- R would be Critical for this cell (in TRANSITION_TABLE), AND
- No other rule in the set fires for this cell with the same target when R is removed

A rule with `unique_critical_cells = 0` is "shadowed Critical" — safe to remove.

```
CompetitionVerdict:
  Preserve:   has unique critical cells (irreplaceable)
  Weakened:   has unique critical cells + high opportunity cost (refine)
  Obsolete:   zero unique critical cells + high opportunity cost
  Neutral:    low cost, no unique critical cells
```

**Canonical spec result**: 0 interference, 0 opportunity cost, all rules Preserve.
`pipeline-advance`: 11/11 cells unique (100% uniqueness).

---

## L12 — SemanticGrowthAnalyzer

**File**: `semantic_growth.rs`  
**Purpose**: Expressiveness Conservation Law — spec must retain capacity for new content

### Semantically Free Cell
A cell (phase, input) is free iff:
1. No rule's guard evaluates to true for it (unclaimed)
2. NOT in TRANSITION_TABLE (no required behavior yet)

### Expressiveness Conservation Law
```
free_fraction(R_{t+1}) >= free_fraction(R_t) × (1 - max_reduction_per_cycle)
```

### SemanticGrowthInvariant (production)
- `min_free_fraction = 0.50` (50% of Q×Σ must stay unclaimed)
- `min_free_cells = 100` (absolute floor)
- `max_reduction_per_cycle = 0.10` (max 10% of free cells claimed per cycle)

**Canonical spec result**: 418/522 = **80.1% free**.
- Most open phase: `ReplayPending`
- Most open input: `RecoverySucceeded` (15 free cells)
- 16/18 phases have semantic headroom
- Growth vector: 418 exact slots, 16 structural phases, 28 structural inputs

---

## The Full EventPipeline

Every event passes through all 5 validation stages. No bypass possible.

```
Event In
    ↓ validate_abi()         ABI major version check
    ↓ validate_schema()      EventSchemaRegistry lookup
    ↓ validate_clock()       lamport monotone + duplicate detection
    ↓ validate_capability()  actor must have required capability grant
    ↓ validate_causal()      causal parent must be processed
    ↓ KernelDecision         Allow | Reject(reason) | Transform
    ↓ StateTransitionFn<E>   reducer applies → new RealmState
    ↓ MaterializerKernel     KernelStamp issued on projection
    ↓ EventLedger.append()   LAST — the commit point
    ↓ KernelPhase::Idle      machine returns to ready
```

**Atomic commit**: if any stage fails → zero observable change.
Observer invariant: `ledger.len() == transition_log.len() == state.version`

---

## ReplayIdentityProof

Not just a test. A structural value:

```rust
pub enum ReplayIdentityProof {
    Confirmed { event_range, final_checksum, transition_count },
    Diverged  { event_range, original_checksum, rebuilt_checksum, diverged_at_lamport },
}
```

`if proof.is_diverged()` → determinism failure → system halt.

---

## Key Files

| Layer | File | Lines |
|---|---|---|
| L0 formal SM | `kernel_state.rs` | 668 |
| L0 constraint algebra | `constraint_algebra.rs` | ~800 |
| L1 machine runner | `kernel_machine.rs` | 478 |
| L2 proof layer | `proof_certificate.rs` | ~600 |
| L3 synthesis | `trace_synthesizer.rs` | ~700 |
| L4 drift | `specification_drift.rs` | ~600 |
| L5 entropy | `specification_entropy.rs` | ~500 |
| L6 stability | `algebra_stability.rs` | ~700 |
| L7 weights | `semantic_weight.rs` | ~600 |
| L8 simplifier | `rule_simplifier.rs` | ~400 |
| L9 counterfactual | `counterfactual.rs` | ~900 |
| L11 competition | `counterfactual_competition.rs` | ~800 |
| L12 growth | `semantic_growth.rs` | ~700 |
| EventPipeline | `pipeline.rs` | 483 |
| StateTransition | `state_transition.rs` | 394 |
| EventLedger | `event_ledger.rs` | 333 |
| Realm (atomic) | `realm.rs` | ~600 |
| Integration proof | `integration.rs` | 192 |

---

*BKG Kernel Formal System · bkg-kernel · 231 tests · 0 clippy warnings*
