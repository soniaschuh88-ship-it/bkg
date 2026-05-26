# BKG — TASKS.md
## Current Status + Roadmap

> **Single source of truth. One module, one location.**
> Last updated: 2026-05

---

## ✅ COMPLETED — All Batches 0–4

### Kernel Formal System (L0–L12) — bkg-kernel
All 12 layers implemented and verified. See [`docs/KERNEL_FORMAL_SYSTEM.md`](KERNEL_FORMAL_SYSTEM.md).

- [x] **L0** `constraint_algebra.rs` — ConstraintExpr, 20 rules, synthesis + verification
- [x] **L0** `kernel_state.rs` — KernelPhase(18), KernelInputKind(29), kernel_delta (TOTAL)
- [x] **L1** `kernel_machine.rs` — KernelMachine runner, TransitionRecord, history
- [x] **L2** `proof_certificate.rs` — ExecutionTrace, ProofChecker (trusted core ~30 lines)
- [x] **L3** `trace_synthesizer.rs` — inductive rule synthesis from traces
- [x] **L4** `specification_drift.rs` — DriftDetector, DriftMonitor, triple-layer checks
- [x] **L5** `specification_entropy.rs` — Shannon + Gini + structural diversity
- [x] **L6** `algebra_stability.rs` — AlgebraInvariant, PinnedRuleSet, SynthesisCycleGuard
- [x] **L7** `semantic_weight.rs` — RuleNecessityProof, causal importance, composite weight
- [x] **L8** `rule_simplifier.rs` — safe Remove/Merge/Generalize operations
- [x] **L9/L10** `counterfactual.rs` — BFS reachability, CounterfactualWitness, SemanticFixationGuard
- [x] **L11** `counterfactual_competition.rs` — DomainInterference, UniqueCriticalCoverage
- [x] **L12** `semantic_growth.rs` — Expressiveness Conservation Law (80.1% free in canonical)

### Architecture Hardening
- [x] **EventPipeline** `pipeline.rs` — validate→decide→apply→emit, all 5 stages
- [x] **bkg-enforce** `cognition/enforce/` — Sealed, InvariantGuard, NoBypass<T>, WorkspaceLints
- [x] **ProjectionView<T>** sealed read-only, no direct RealmState access outside bkg-state
- [x] **TypedEvent<P: EventPayload>** — 9 canonical types, compile-time schemas
- [x] **Projection Hash Contract** — EventRange, ProjectionChecksum, MaterializerKernel, KernelStamp
- [x] **StateTransitionFn<E>** + ReplayIdentityProof as structural invariant
- [x] **Realm** (atomic) — single atomic commit, zero dual-truth drift
- [x] **EventLedger** — append-only, BLAKE3 hash-chained, tamper-evident

### Batch 0 — Architecture Foundation
- [x] `bkg-state` — RealmState, Reducer<E>, ProjectionView<T>, mutation, invariants, reconciliation
- [x] `bkg-abi` — AbiEnvelope<T>, 7 typed ABIs (event, packet, capsule, projection, plugin, provider, mesh)
- [x] `bkg-clock` — SequencedInstant, VectorClock, no SystemTime::now()
- [x] `bkg-schema` — EventSchemaRegistry, migration strategies, schema versioning
- [x] `DomainEvent<T>` — TypedEvent<P: EventPayload>, 9 canonical event types
- [x] kernel arbitrator — KernelArbitrator, causality judge
- [x] workflow ExecutionGraph — loops, retries, parallel waves

### Batch 1 — Core Application
- [x] `bkg-project` — project registry, settings, 5 model lanes
- [x] `bkg-workflow` — Plan→Review→Execute, verdicts, wave execution
- [x] `bkg-query` — BQL engine: parser, AST, executor, planner
- [x] `bkg-task` — task capsules, lifecycle SM, DAG, T-ID
- [x] `bkg-mission` — Mission→Milestone→Slice→Task, autopilot
- [x] `bkg-scheduler` — deterministic DAG, priority queue, overlap gating
- [x] `bkg-lanes` — Realm Bus IPC, 4 priority classes, backpressure

### Batch 1.5 — ECS Foundation
- [x] `bkg-ecs` — deterministic sparse-archetype ECS (18 tests)
- [x] `bkg-projection` — ProjectionCache, Materializer, ProjectionIndex
- [x] `bkg-identity` — DeterministicId::derive, AncestryChain, RealmIdentity

### Batch 2 — Security + Features
- [x] `bkg-secrets` — AES-256-GCM store, scopes, policies, env export
- [x] `bkg-approval` — ApprovalGate, immutable ApprovalAudit, double-decide protection
- [x] `bkg-capabilities` — CapabilitySet, CapabilityGrant (TTL+revocable), ExecutionScope
- [x] `bkg-eval` — Scorecard (weighted bands A-F), EvalEvidence, EvalBatch
- [x] `bkg-chat` — ChatRoom, ChatMessage (mentions), Mailbox
- [x] `bkg-github` — GithubAuth, GithubIssue, PullRequest (Squash/Merge/Rebase)
- [x] `bkg-plugins` — PluginManifest, PluginRegistry, PluginLoader

### Batch 3 — Infrastructure
- [x] `bkg-mesh` — MeshNode, LeaseRegistry (epoch-fenced), SyncRecord, NodeRegistry
- [x] `bkg-vm` — SandboxVm (mounts, env, snapshot, seal), SyscallFilter
- [x] `bkg-snapshot` — RealitySnapshot (checksum, fork, gc_eligible), RealmSnapshot
- [x] `bkg-migration` — VersionMap, MigrationPlan, MigrationRunner (Apply/Skip/Fail)

### Batch 4 — Advanced Systems
- [x] `bkg-physics` — PhysicsNode, SpringForce, PhysicsSimulation, system_entropy
- [x] `bkg-entropy` — MetricSnapshot (entropy/pressure/heat/stability), SystemMetrics
- [x] `bkg-compiler` — UiAst, UiCompiler (deterministic), Bytecode, UiFrame
- [x] `bkg-render` — HeadlessBackend, ANSI renderer
- [x] `bkg-diff` — StateDiff, GraphDiff, CausalTrace
- [x] `bkg-recovery` — CrashClassification, RepairStrategy, RecoveryCheckpoint
- [x] `bkg-gc` — GcPolicy, GcPressure (5 levels), GcRun (compact)
- [x] `bkg-lineage` — LineageGraph, ForkRecord, common_ancestor
- [x] `bkg-simulation` — SimWorld, SimAgent, Oracle assertions
- [x] `bkg-world` — WorldGraph, World (versioned), CausalChain
- [x] `bkg-operator` — OperatorIntent, AttentionMap (decay), InteractionHistory

---

## 🔄 NEXT — Batch 5 + Integration

### Consensus
- [ ] `bkg-consensus` — Raft-inspired mesh arbitration

### Integration passes
- [ ] Wire `Realm` into domain crates (bkg-task, bkg-workflow etc. use real Realm)
- [ ] Wire `SemanticGrowthAnalyzer` into `SynthesisCycleGuard`
- [ ] Wire `CounterfactualCompetitionLayer` into `RuleSimplifier`
- [ ] Integrate `DriftMonitor` into `Realm::submit_event()`
- [ ] `bkg-world` full integration — connect ECS + physics + BQL + projections
- [ ] `bkg-atlantean` Kanban board — reads from ProjectionView<KanbanProjection>

### Testing
- [ ] Integration tests across crate boundaries
- [ ] Replay identity tests with real domain events
- [ ] Fuzz testing for EventPipeline rejection paths

### Documentation
- [ ] API docs (cargo doc --workspace)
- [ ] Architecture decision records (ADRs)

---

## Test Coverage Summary

| Crate | Tests |
|---|---|
| bkg-kernel | 231 |
| bkg-state | 33 |
| bkg-event | 20 |
| bkg-enforce | 15 |
| bkg-ecs | 18 |
| bkg-projection | 9 |
| bkg-identity | 12 |
| bkg-mesh | 9 |
| bkg-lanes | 17 |
| bkg-snapshot | 4 |
| bkg-migration | 5 |
| bkg-physics | 6 |
| bkg-entropy | 3 |
| bkg-compiler | 4 |
| bkg-render | 2 |
| bkg-simulation | 3 |
| bkg-secrets | 6 |
| bkg-approval | 3 |
| bkg-capabilities | 6 |
| bkg-eval | 3 |
| bkg-chat | 3 |
| bkg-github | 1 |
| bkg-plugins | 3 |
| bkg-gc | 2 |
| bkg-lineage | 1 |
| all other crates | ~30 |

**Total**: 231 in bkg-kernel alone; 400+ across workspace.

---

*BKG v0.1.0 · DELPHOS · Batches 0–4 complete · 231 kernel tests*
