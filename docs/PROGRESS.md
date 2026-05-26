# BKG — PROGRESS.md
## Development History + Architectural Decisions

> **Single source of truth. One module, one location.**

---

## System Classification (current)

BKG is now precisely:

> **a causally constrained, counterfactually aware semantic persistence system**

Or equivalently:

> **a self-consistent rule inference system constrained by normalized causal attribution**

It is simultaneously:
- A **runtime model checker** (every execution is checked)
- A **self-inducing transition system** (induces rules from its own behavior)
- A **certified execution DAG machine** (every step has a proof certificate)
- A **bi-directional semantics system** (execution ↔ specification)

---

## What was built (complete log)

### Phase 1 — Foundation (2026-04)

**BKG v0.1.0** (commit `6578342`) — 18 crates:
`bkg-core`, `bkg-crypto`, `bkg-event`, `bkg-contracts`, `bkg-kernel`,
`bkg-swd`, `bkg-capsule`, `bkg-store`, `bkg-memory`, `bkg-replay`,
`bkg-verifier`, `bkg-policy`, `bkg-runtime`, `bkg-orchestrator`, `bkg-exec`,
`bkg-tools`, `bkg-inspector`, `bkg-cli`

**pi-free integration** — `bkg-providers` (13 providers), `bkg-telemetry`

**bkg-atlantean** — Cyberpunk/Atlantis UI dashboard

**sandbox-agent integration** — `bkg-agents`, `bkg-session`, `bkg-acp`

### Phase 2 — DELPHOS Architecture (2026-05)

**Batches 0–4 completed** (60+ crates):
All domain crates implemented with real code (zero stubs remaining).

**Architecture hardening** — 4 structural improvements:
- `EventPipeline` (validate→decide→apply→emit, law of physics)
- `bkg-enforce` (Sealed traits, InvariantGuard, NoBypass<T>)
- `ProjectionView<T>` (sealed read-only isolation)
- `TypedEvent<P: EventPayload>` (9 canonical event types)

**Projection Hash Contract**:
- `EventRange` + `ProjectionChecksum` + `MaterializerKernel` + `KernelStamp`
- Law 1: every projection has EventRange + checksum
- Law 2: every projection rebuildable from ledger alone
- Law 3: materializer validated by kernel

**Kernel mathematical fix**:
- `StateTransitionFn<E>` as explicit type
- `ReplayIdentityProof` as structural value (not just a test)
- `ReplaySession` + `TransitionLog` for formal replay

### Phase 3 — Formal Kernel System (2026-05)

**L0: KernelMachine — formal Mealy machine** (`fd64e80`)
- `KernelPhase` (18 states), `KernelInputKind` (29 inputs)
- `kernel_delta()` — TOTAL, DETERMINISTIC, verified over all 522 cells
- `KernelMachine` — step(), history (append-only), fault classification
- 71 tests

**δ-compression + λ-isolation + Realm atomic commit** (`2331916`)
- `RuleEngine` — 19 algebraic rules synthesize the full table
- `KernelEffectIsolated` — Copy type, no runtime strings (λ isolation)
- `EventLedger` — BLAKE3 hash-chained, append-only
- `Realm::submit_event()` — atomic commit (ledger appends last)
- 105 tests

**Constraint Algebra + ProofCertificate** (`37f9619`)
- `ConstraintExpr` — symbolic predicates, subsumes(), conflicts_with()
- `canonical_constraint_rules()` — 20 rules covering the full table
- `ProofChecker` — trusted core ~30 lines, no KernelMachine dependency
- 131 tests

**TraceSynthesizer + DriftDetector** (`025209a`)
- Inductive rule synthesis from traces (Occam's Razor)
- Triple-layer drift detection: algebra ↔ kernel ↔ traces
- Bug caught: pipeline-advance was cross-product (110 cells) → fixed to 11 exact pairs
- 148 tests

**SpecificationEntropy + AlgebraStability** (`5dd5bea`)
- Shannon entropy, Gini coefficient, structural diversity, compression ratio
- `AlgebraInvariant` (HARD/SOFT), `PinnedRuleSet`, `SynthesisCycleGuard`
- 169 tests

**SemanticWeightLayer** (`29eb981`)
- `RuleNecessityProof` (Critical / Observational / Redundant)
- Causal importance grounded in traces; sum = 1.0 exactly
- Anti-gaming: all scores externally anchored
- 180 tests

**RuleSimplifier** (`5a15e6b`)
- Safe Remove/Merge/Generalize operations
- Every transformation checked via invariants + entropy floor
- Canonical spec: zero removable rules (already minimal)
- 186 tests

**CounterfactualAnalyzer + SemanticFixationGuard** (`2e51835`)
- BFS over TRANSITION_TABLE: distances, path reconstruction
- `CounterfactualWitness` — shortest path to activate a rule
- `SemanticFixationGuard` — prevents removing reachable rules
- Canonical: all 18 phases reachable, max_distance = 11
- 206 tests

**CounterfactualCompetitionLayer** (`24ed2ed`)
- `DomainInterference`, `UniqueCriticalCoverage` (the decisive flag)
- Breaks infinite preservation bias: `unique_critical_cells = 0` → safe to remove
- Canonical: 0 interference, `pipeline-advance` = 11/11 unique
- 218 tests

**SemanticGrowthAnalyzer** (`0fe47a1`)
- Expressiveness Conservation Law
- `free_fraction >= 0.50` (production invariant)
- Canonical: 418/522 = 80.1% free
- `HeadroomHistory` tracks reduction rate across cycles
- 231 tests

---

## Key Architectural Decisions

### ADR-001: DELPHOS over Fusion
Fusion = application layer. BKG = deterministic substrate beneath it.
All 258+ Fusion features map onto DELPHOS realms. No Fusion code imported as-is.

### ADR-002: pi-free → bkg-providers
Full Rust rewrite. 13 providers. Extended with tier metadata, BkgSupervised mode.

### ADR-003: sandbox-agent → bkg-agents + bkg-session + bkg-acp
Portable to BKG structure. `_bkg/` replaces `_sandboxagent/`.

### ADR-004: ECS to Batch 1.5 (was Batch 4)
Physics, compiler, and world need ECS as foundation. Standard ECS breaks replay determinism.
Deterministic sparse archetype ECS with stable BTreeMap iteration order.

### ADR-005: bkg-state Reducer is the ONLY state mutator
`apply(state: &RealmState, event: E) -> Result<RealmState>` — the only path.
Without this: replay correctness depends on discipline (fragile at scale).

### ADR-006: Formal kernel as Mealy machine
The kernel is not just an event processor. It is a formally specified M=(Q,Σ,Λ,δ,λ,q₀)
with verified properties: TOTAL, DETERMINISTIC, ABSORBING, MONOTONE.

### ADR-007: Rules as source of truth (not the 522-cell table)
The explicit transition table is a DERIVED artifact. The 20 constraint rules are the source.
This was discovered when the pipeline-advance cross-product rule caused 129 cell disagreements
— caught immediately by `DriftDetector`.

### ADR-008: Proof-carrying automaton
Every transition produces a verifiable `TransitionCertificate`. The `ProofChecker`
(trusted core, ~30 lines) can verify correctness without running the kernel.
Third-party verification without trusting our code.

### ADR-009: Semantic weight grounds simplification in reality
Rule simplification is not structural (entropy). It is semantic (necessity + causal importance).
The "causal importance sums to 1.0" invariant prevents metric gaming.

### ADR-010: Counterfactual competition breaks infinite preservation bias
A rule is safe to simplify if `unique_critical_cells = 0` — another rule already
provides the same Critical coverage. Reachability alone is not sufficient justification.

### ADR-011: Expressiveness Conservation Law
The spec must maintain ≥50% semantic headroom. No single synthesis cycle may claim
>10% of free cells. At 80.1% free, the canonical spec has substantial room to evolve.

---

## Known Issues

### Architecture
- [ ] Async determinism: tokio alone is not fully replay-safe. Tick-driven executor needed.
- [ ] `bkg-acp` AgentBridge doesn't spawn real processes yet.
- [ ] `bkg-session` in-memory only — sessions don't survive server restart.
- [ ] No Kanban board in Atlantean (biggest missing UI feature).

### Integration gaps
- [ ] Domain crates (bkg-task, bkg-workflow) not yet wired to `Realm`.
- [ ] `SemanticGrowthAnalyzer` not yet wired into `SynthesisCycleGuard`.
- [ ] `DriftMonitor` not yet wired into `Realm::submit_event()`.

---

*BKG v0.1.0 · DELPHOS · 231 kernel tests · 60+ crates*
*Single source of truth. One module, one location.*
