# BKG — Project Organisation Rules

> Single source of truth. One module, one location.

---

## Core Principles

1. **One concept, one crate** — no concept may be split across crates
2. **Single mutation path** — all state changes via `StateTransitionFn<E>` only
3. **EventPipeline required** — all events must pass through `EventPipeline.process()`
4. **Replay-safe** — same ledger + same reducers = same final state, always
5. **No SystemTime::now()** — use `bkg-clock` `SequencedInstant` for all ordering
6. **Projection-only UI** — UI reads `ProjectionView<T>` only, never `RealmState`

---

## Directory Map (complete)

### Cognition — System Law
```
delphos/cognition/core/         bkg-core         Primitive types, IDs, errors
delphos/cognition/kernel/       bkg-kernel        Formal kernel M=(Q,Σ,Λ,δ,λ,q₀) L0–L12
delphos/cognition/event/        bkg-event         TypedEvent<P>, DomainEvent, EventLedger
delphos/cognition/state/        bkg-state         RealmState, ProjectionView<T>, Reducer
delphos/cognition/abi/          bkg-abi           AbiEnvelope<T>, 7 typed ABIs
delphos/cognition/clock/        bkg-clock         SequencedInstant, VectorClock
delphos/cognition/enforce/      bkg-enforce       Sealed, InvariantGuard, NoBypass<T>
delphos/cognition/schema/       bkg-schema        EventSchemaRegistry, migrations
delphos/cognition/contracts/    bkg-contracts     CausalContract types
delphos/cognition/protocol/     bkg-acp           ACP JSON-RPC 2.0, AgentBridge
delphos/cognition/project/      bkg-project       Project registry + settings
delphos/cognition/workflow/     bkg-workflow      Plan→Review→Execute + ExecutionGraph
delphos/cognition/query/        bkg-query         BQL engine
```

### Thalassa — Execution Realm
```
delphos/domains/thalassa/runtime/       bkg-runtime       AgentRuntime
delphos/domains/thalassa/orchestrator/  bkg-orchestrator  TaskGraph, EventBus
delphos/domains/thalassa/providers/     bkg-providers     13 LLM providers
delphos/domains/thalassa/agents/        bkg-agents        7 agents
delphos/domains/thalassa/session/       bkg-session       UniversalEvent, BkgSession
delphos/domains/thalassa/exec/          bkg-exec          bash, file, grep executors
delphos/domains/thalassa/task/          bkg-task          Task capsules + lifecycle
delphos/domains/thalassa/mission/       bkg-mission       Mission→Milestone→Task
delphos/domains/thalassa/scheduler/     bkg-scheduler     Deterministic DAG scheduler
delphos/domains/thalassa/chat/          bkg-chat          ChatRoom, Mailbox
delphos/domains/thalassa/github/        bkg-github        Issue import, PR creation
delphos/domains/thalassa/simulation/    bkg-simulation    SimWorld, SimAgent, Oracle
delphos/domains/thalassa/vm/            bkg-vm            SandboxVm, SyscallFilter
```

### Arche — Persistence Realm
```
delphos/domains/arche/capsule/  bkg-capsule   Capsule + lifecycle SM
delphos/domains/arche/store/    bkg-store     sled + in-memory
delphos/domains/arche/mesh/     bkg-mesh      MeshNode, LeaseRegistry
```

### Styx — Event Provider Realm
```
delphos/domains/styx/provider/  bkg-swd     SwdEngine
delphos/domains/styx/tools/     bkg-tools   ledger_summary
delphos/domains/styx/lanes/     bkg-lanes   RealmBus, LaneClass, LaneRouter
```

### Katoptron — Observation + Projection Realm
```
delphos/domains/katoptron/crypto/       bkg-crypto      BLAKE3, Ed25519
delphos/domains/katoptron/verifier/     bkg-verifier    hash-chain, PermissionEnforcer
delphos/domains/katoptron/telemetry/    bkg-telemetry   model call tracking
delphos/domains/katoptron/approval/     bkg-approval    ApprovalGate, ApprovalAudit
delphos/domains/katoptron/secrets/      bkg-secrets     SecretsStore, AES-256-GCM
delphos/domains/katoptron/eval/         bkg-eval        Scorecard, EvalEvidence
delphos/domains/katoptron/plugins/      bkg-plugins     PluginRegistry, PluginLoader
delphos/domains/katoptron/ecs/          bkg-ecs         World (BTreeMap stable order)
delphos/domains/katoptron/projection/   bkg-projection  ProjectionCache, Materializer
delphos/domains/katoptron/physics/      bkg-physics     PhysicsSimulation
delphos/domains/katoptron/compiler/     bkg-compiler    UiAst → Bytecode
delphos/domains/katoptron/entropy/      bkg-entropy     MetricSnapshot
delphos/domains/katoptron/render/       bkg-render      HeadlessBackend, ANSI
delphos/domains/katoptron/world/        bkg-world       WorldGraph, CausalChain
```

### Anamnesis — Policy + Memory Realm
```
delphos/domains/anamnesis/policy/       bkg-policy    PolicyEngine
delphos/domains/anamnesis/operator/     bkg-operator  OperatorIntent, AttentionMap
```

### Mnemos — Replay Realm
```
delphos/domains/mnemos/memory/  bkg-memory   MemoryGraph
delphos/domains/mnemos/replay/  bkg-replay   ReplayEngine
```

### Speculum — Verification + Audit Realm
```
delphos/domains/speculum/capabilities/  bkg-capabilities  CapabilitySet, CapabilityGrant
delphos/domains/speculum/snapshot/      bkg-snapshot      RealitySnapshot
delphos/domains/speculum/diff/          bkg-diff          StateDiff, CausalTrace
delphos/domains/speculum/recovery/      bkg-recovery      CrashClassification
delphos/domains/speculum/gc/            bkg-gc            GcPolicy, GcPressure
delphos/domains/speculum/identity/      bkg-identity      DeterministicId
delphos/domains/speculum/lineage/       bkg-lineage       LineageGraph
delphos/domains/speculum/migration/     bkg-migration     VersionMap, MigrationRunner
```

### UI + CLI
```
delphos/reflection/ui/atlantean/  bkg-atlantean  Cyberpunk/Atlantis dashboard
delphos/threshold/cli/            bkg-cli        `bkg` binary
```

---

## Invariants

### Must always hold
- `cargo check --workspace` — zero errors
- `cargo clippy --workspace -- -D warnings` — zero warnings
- `cargo test --workspace` — all tests pass
- `DriftDetector::canonical_algebra_agrees_with_kernel_everywhere` — 100% alignment

### Code invariants
- No `allow(dead_code)` in committed code (zero stubs)
- No `SystemTime::now()` outside display code
- No direct `RealmState` mutation outside `StateTransitionFn<E>`
- No `ProjectionView<T>` construction outside `ProjectionFactory` (pub(crate))
- All events must pass through `EventPipeline.process()` before `Realm::submit_event()`

### Architectural invariants
- Cross-realm access only via `CausalContract` + `RealmRouter`
- UI reads `ProjectionView<T>` only — never the ledger or `RealmState`
- Replay: `fold(f, S0, [e1..en]) = Sn` always (structural, not tested)
- Ledger appends LAST in `Realm::submit_event()` (commit point)
- `ledger.len() == transition_log.len() == state.version` (consistency invariant)

---

## Semantic Growth Constraint

The kernel specification must maintain ≥50% free semantic space.

Current: **80.1% free** (418/522 cells unclaimed)

A synthesis cycle may not reduce free cells by more than 10% in one step.
Any rule set claiming >50% of Q×Σ fails the `SemanticGrowthInvariant.production()` check.

---

*BKG v0.1.0 · DELPHOS · Single source of truth. One module, one location.*
