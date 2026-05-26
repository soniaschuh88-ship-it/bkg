# BKG — task.md · Full DELPHOS Specification v2

> **System ethic**: *Single source of truth. One module, one location.*
> **Vision**: BKG is not a task app. It is a deterministic OS for causal agent orchestration
> with replayable reality simulation.

---

## Ontology

```
Event Ledger  →  Reducer  →  World Graph  →  Physics  →  Compiler  →  Projection  →  UI
```

The UI is the visible shadow of event causality.
Nothing exists outside the event ledger. No UI state. No hidden mutations.

---

## Current Status

**24 crates, 130+ Rust files, 7 git commits** — foundation solid.
See `TASK.md` for current status, `FEATURES.md` for full feature inventory, `README.md` for setup.

---

## Critical Architecture Gaps (from analysis)

The following 12 structural issues must be resolved before application features.
Without these, the system drifts from "agent framework" toward unrecoverable chaos.

### Gap 1 — RealmStateMachine MISSING ⚠️ CRITICAL
Currently multiple crates mutate state independently. No single canonical reducer.
Fix: **`bkg-state`** — `apply(state, event) -> Result<RealmState>` is the ONLY mutator.
UI reads ONLY Projections. Never the ledger directly.
Modules: `reducer.rs`, `projection.rs`, `realm_state.rs`, `transition.rs`, `mutation.rs`, `snapshot.rs`, `reconciliation.rs`, `invariants.rs`

### Gap 2 — No canonical Entity model ⚠️ CRITICAL
Tasks, missions, agents, sessions have no shared ontological base.
Without ECS: physics, compiler, diff, UI graph, and mesh replication all explode.
Fix: **`bkg-ecs`** moved to **Batch 1.5** (was Batch 4 — too late).
ALL DELPHOS entities become ECS entities: Task, Mission, Slice, Agent, Session, Approval, Message, WorkflowStep, Provider, MeshNode, Timeline.

### Gap 3 — No Global Deterministic World Snapshot ⚠️ CRITICAL
Capsule snapshots exist, but no full `GenesisSnapshot` / `RealitySnapshot` / `TimelineSnapshot`.
Fix: **`bkg-snapshot`** (`speculum/snapshot`) — fork/export/restore full world state.
CLI: `bkg snapshot create/fork/export/diff/restore`

### Gap 4 — Kernel too passive ⚠️ HIGH
RealmRouter + Validator + Genesis exist, but Kernel doesn't enforce causality integrity.
Fix: **`kernel/arbitrator.rs`** — prevents concurrent causality corruption, invalid realm transitions, duplicate tick chains, cyclic approvals, replay paradoxes.
Kernel becomes: BIOS + Hypervisor + Causality Judge.

### Gap 5 — Event typing too flat ⚠️ HIGH
Current `EventType` enum is untyped `serde_json::Value` payload.
Fix: **`DomainEvent<T>`** in `bkg-event` — compile-time replay validation, typed reducers, typed projections, event ABI, mesh sync verification.

### Gap 6 — No Universal Realm ABI ⚠️ CRITICAL
Without ABI, every crate speaks slightly different serialization → "alles spricht leicht andere Sprache" → architecture death.
Fix: **`bkg-abi`** (`cognition/abi`) — standardizes ALL inter-system communication:
events · packets · capsules · projections · UI bytecode · mesh sync · plugins · providers

### Gap 7 — Projection Cache missing ⚠️ HIGH
UI cannot read the full ledger every render cycle. Need materialized read models.
Fix: **`bkg-projection`** (`katoptron/projection`) — materializer, index, invalidation, realtime subscriptions.
Atlantean, physics, kanban, scheduler all read ONLY from projections.

### Gap 8 — Capsule has no lifecycle state machine ⚠️ HIGH
Currently capsules are created/versioned but have no formal state transitions.
Fix: extend **`bkg-capsule`** with `Created → Mounted → Active → Frozen → Forked → Archived → Corrupted → Recovered`.
Critical for Mesh + Replay + VM correctness.

### Gap 9 — Workflow Engine has no formal ExecutionGraph ⚠️ HIGH
DAG ≠ WorkflowGraph. Workflow needs: loops, retries, fallback branches, parallel waves, conditional transitions.
Fix: **`workflow/graph.rs`** in the upcoming `bkg-workflow` implementation.

### Gap 10 — No BKG Query Language ⚠️ MEDIUM (later)
Without BQL, UI filters, telemetry queries, physics queries, AI context extraction all become SQL/ECS chaos.
Fix: **BQL** (`bkg-world` includes query layer):
`SELECT tasks WHERE status = "blocked" AND dependency.depth > 3 ORDER BY entropy DESC`

### Gap 11 — Realm DNA missing ⚠️ HIGH
Realm isolation is currently "soft" — any realm could theoretically accept any event.
Fix: **RealmDNA** (`cognition/realm-dna`) — each realm declares: allowed events, allowed mutations, allowed capabilities, allowed lanes, allowed clocks, allowed reducers.

### Gap 12 — No deterministic memory allocation ⚠️ LATER
For true `same seed + same ledger = same output`, need deterministic: task ordering, allocator behavior, scheduler timing, async polling.
tokio alone is insufficient for full determinism. Fix: deterministic executor + replay scheduler + tick-driven async. (Long-term research task.)

---

## Updated Batch Order

### Batch 0 — Foundation (IMMEDIATE — before any app features)
```
bkg-state           cognition/state         RealmStateMachine, reducer, projections
bkg-abi             cognition/abi           Universal Realm ABI
bkg-clock           cognition/clock         Vector clocks, causal ordering, no SystemTime::now()
kernel/arbitrator   extend bkg-kernel       Causality judge, replay paradox prevention
DomainEvent<T>      extend bkg-event        Typed events for compile-time validation
workflow/graph      in bkg-workflow impl    ExecutionGraph: loops, waves, conditionals
```

### Batch 1 — Core Application
```
bkg-task            domains/thalassa/task   Task capsules + lifecycle + DAG
bkg-project         cognition/project       Project registry + settings
bkg-workflow        cognition/workflow      Plan→Review→Execute + verdicts
bkg-scheduler       domains/thalassa/scheduler  Deterministic DAG scheduler
bkg-mission         domains/thalassa/mission    Mission hierarchy + autopilot
bkg-lanes           domains/styx/lanes      Realm Bus IPC fabric
```

### Batch 1.5 — ECS Foundation (moved UP from Batch 4)
```
bkg-ecs             domains/katoptron/ecs   Entity Component System (world foundation)
bkg-projection      domains/katoptron/projection  ProjectionCache + materializer
realm-dna           cognition/realm-dna     RealmDNA per realm
capsule lifecycle   extend bkg-capsule      Created/Mounted/Active/Frozen/Forked/Archived
```

### Batch 2 — Security + Features
```
bkg-secrets         domains/katoptron/secrets    AES-256-GCM + OS keychain
bkg-approval        domains/katoptron/approval   Gates + audit trail
bkg-capabilities    domains/speculum/capabilities Signed execution scopes
bkg-eval            domains/katoptron/eval       Scorecards + evidence
bkg-chat            domains/thalassa/chat        Rooms + mailbox + SSE
bkg-github          domains/thalassa/github      Issue import + PR creation
bkg-plugins         domains/katoptron/plugins    YAML manifest + loader
```

### Batch 3 — Infrastructure
```
bkg-mesh            domains/arche/mesh       Multi-node replication + leases
bkg-vm              domains/thalassa/vm      Sandbox VM + syscall layer
bkg-snapshot        domains/speculum/snapshot World snapshots (fork/export/restore)
```

### Batch 4 — Advanced Systems
```
bkg-physics         domains/katoptron/physics  DAG physics: mass, tension, entropy
bkg-compiler        domains/katoptron/compiler Ledger → AST → Geometry → Bytecode
bkg-diff            domains/speculum/diff      Reality diff engine
bkg-recovery        domains/speculum/recovery  Crash reconstruction
bkg-operator        domains/anamnesis/operator Operator consciousness
bkg-world           domains/katoptron/world    ULTIMATE: Causal World Model + BQL
```

### Batch 5 — UI + CLI
```
atlantean: Kanban, Task detail, Missions, Physics, Diff, Chat, Secrets, Approvals, Mesh
cli:       task/mission/project/workflow/secrets/eval/mesh/snapshot commands
terminal:  ratatui backend, headless CI backend
```

---

## Full DELPHOS Layout (42 crates target)

```
bkg/
└── delphos/
    ├── cognition/
    │   ├── core/         bkg-core         ✅  IDs, Hash256, BkgError
    │   ├── kernel/       bkg-kernel       ✅+ Genesis, RealmRouter, Arbitrator(📋)
    │   ├── event/        bkg-event        ✅+ Event, Ledger, DomainEvent<T>(📋)
    │   ├── contracts/    bkg-contracts    ✅  CausalContract
    │   ├── protocol/     bkg-acp          ✅  ACP JSON-RPC, AgentBridge
    │   ├── state/        bkg-state        📋  CRITICAL — RealmStateMachine
    │   ├── abi/          bkg-abi          📋  CRITICAL — Universal ABI
    │   ├── clock/        bkg-clock        📋  CRITICAL — Vector clocks
    │   ├── project/      bkg-project      📋  Project registry
    │   └── workflow/     bkg-workflow     📋  Plan→Review→Execute
    │
    └── domains/
        ├── thalassa/
        │   ├── runtime/      bkg-runtime      ✅
        │   ├── orchestrator/ bkg-orchestrator ✅
        │   ├── providers/    bkg-providers    ✅  (pi-free)
        │   ├── agents/       bkg-agents       ✅  (sandbox-agent)
        │   ├── session/      bkg-session      ✅  (sandbox-agent)
        │   ├── exec/         bkg-exec         ✅
        │   ├── task/         bkg-task         📋  Task capsules
        │   ├── mission/      bkg-mission      📋  Mission hierarchy
        │   ├── scheduler/    bkg-scheduler    📋  DAG scheduler
        │   ├── chat/         bkg-chat         📋  Chat rooms
        │   ├── github/       bkg-github       📋  GitHub integration
        │   └── vm/           bkg-vm           📋  Sandbox VM
        ├── arche/
        │   ├── capsule/  bkg-capsule      ✅+  (lifecycle SM 📋)
        │   ├── store/    bkg-store        ✅
        │   └── mesh/     bkg-mesh         📋  Multi-node
        ├── styx/
        │   ├── provider/ bkg-swd          ✅
        │   ├── tools/    bkg-tools        ✅
        │   └── lanes/    bkg-lanes        📋  Realm Bus
        ├── katoptron/
        │   ├── crypto/   bkg-crypto       ✅
        │   ├── verifier/ bkg-verifier     ✅
        │   ├── telemetry/ bkg-telemetry   ✅+  (physics 📋)
        │   ├── approval/ bkg-approval     📋
        │   ├── secrets/  bkg-secrets      📋
        │   ├── eval/     bkg-eval         📋
        │   ├── plugins/  bkg-plugins      📋
        │   ├── ecs/      bkg-ecs          📋  CRITICAL (Batch 1.5)
        │   ├── projection/ bkg-projection 📋
        │   ├── physics/  bkg-physics      📋
        │   ├── compiler/ bkg-compiler     📋
        │   └── world/    bkg-world        📋  ULTIMATE
        ├── anamnesis/
        │   ├── policy/   bkg-policy       ✅
        │   ├── memory/   bkg-memory       ✅  (dreams, insights)
        │   └── operator/ bkg-operator     📋
        ├── mnemos/
        │   ├── memory/   bkg-memory       ✅
        │   └── replay/   bkg-replay       ✅
        └── speculum/
            ├── capabilities/ bkg-capabilities 📋
            ├── snapshot/     bkg-snapshot     📋
            ├── diff/         bkg-diff         📋
            └── recovery/     bkg-recovery     📋

    └── reflection/
        └── ui/
            └── atlantean/    bkg-atlantean    ✅+  (Kanban, Tasks, Missions 📋)
    └── threshold/
        └── cli/              bkg-cli          ✅+  (Fusion commands 📋)
    └── calibration/
        └── testing/          bkg-testing      ✅
```

---

## Causal Data Flow (target)

```
Event Ledger (bkg-event)
    ↓ apply() — bkg-state (reducer)
RealmState
    ↓ RealmDNA validation
Validated State
    ↓ entity sync — bkg-ecs
World Graph (bkg-world)
    ↓ physics — bkg-physics
Geometry
    ↓ compile — bkg-compiler
Render Bytecode
    ↓ projection cache — bkg-projection
Atlantean UI / Terminal (ratatui)
```

---

## Fusion Features Mapped (258+ features)

Full mapping in `FEATURES.md`.

**Quick summary by status:**

| Group | Done | Partial | Planned |
|---|---|---|---|
| Core Foundation | ✅ 4 crates | — | 📋 3 crates |
| Execution Realm | ✅ 6 crates | — | 📋 5 crates |
| Persistence | ✅ 2 crates | — | 📋 2 crates |
| Observation/Projection | ✅ 3 crates | — | 📋 9 crates |
| Policy/Memory | ✅ 2 crates | — | 📋 1 crate |
| Replay | ✅ 2 crates | — | — |
| Verification | — | — | 📋 4 crates |
| IPC Fabric | — | — | 📋 1 crate |
| Dashboard | ✅ partial | — | 📋 extend |
| CLI | ✅ partial | — | 📋 extend |

---

## sandbox-agent Features Ported

| Feature | Status |
|---|---|
| Universal agent support (7 agents) | ✅ `bkg-agents` |
| Universal event schema | ✅ `bkg-session` |
| Session lifecycle management | ✅ `bkg-session` |
| SSE event streaming | ✅ `bkg-session` + `bkg-atlantean` |
| Permission handling (human-in-loop) | ✅ `bkg-session` |
| ACP JSON-RPC 2.0 | ✅ `bkg-acp` |
| Agent Bridge (stdout → UniversalEvent) | ✅ `bkg-acp` |
| Inspector UI | ✅ `bkg-atlantean` |
| Offset-based event replay | ✅ `bkg-session` |
| Process runtime | 📋 `bkg-vm` |
| File system operations | ✅ `bkg-exec` |
| Tool execution sandbox | 📋 `bkg-vm` |

---

## pi-free Features Ported

| Feature | Status |
|---|---|
| 13 providers (all tiers) | ✅ `bkg-providers` |
| Free-model detection (Route A+B) | ✅ `bkg-providers` |
| Per-provider toggle (persisted) | ✅ `bkg-providers` |
| CI score enhancer | ✅ `bkg-providers` |
| Model call telemetry | ✅ `bkg-telemetry` |
| Quota monitoring | ✅ `bkg-telemetry` |
| Provider fallback chain | ✅ everywhere |
| Admin global keys | ✅ `bkg-atlantean` |
| User-level provider keys | ✅ `bkg-atlantean` |
| Onboarding wizard | ✅ `bkg-atlantean` |
| Model ABI layer | 📋 `providers/abi` |

---

*BKG v0.1.0 · DELPHOS architecture*
*Target: 42 crates · 258+ Fusion features · 15 new core systems · 12 architecture gaps resolved*
*Single source of truth. One module, one location.*
