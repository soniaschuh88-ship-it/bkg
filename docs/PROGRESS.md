# BKG — task.md · DELPHOS Specification v3

> **Single source of truth. One module, one location.**
> **BKG is a deterministic ontology engine. AI agents are inhabitants, not the core.**

---

## The Final Vision

```
Not: "AI agent framework"
But: "deterministic causal operating substrate with replayable world simulation"
```

Everything in DELPHOS is a projection of the same causal world.
Physics, UI, AI planning, telemetry, mission orchestration — all different views of one truth.

---

## Causal Data Flow (target)

```
DomainEvent<T>  (typed, signed, causal parent, schema_id)
    ↓
bkg-event       (append-only ledger, BLAKE3 hash chain)
    ↓ apply() — Reducer<E>
bkg-state       (RealmState — immutable, copy-on-write)
    ↓ RealmDNA validation
bkg-kernel      (Arbitrator — causality judge)
    ↓ entity sync
bkg-ecs         (deterministic sparse archetype world)
    ↓
bkg-world       (RealityGraph — entities + relations + causality)
    ↓ n-body physics
bkg-physics     (geometry)
    ↓ compile via bkg-abi bytecode
bkg-compiler    (render bytecode)
    ↓ materializer
bkg-projection  (indexed, checksummed, rebuildable read models)
    ↓ BQL queries
bkg-query       (BQL engine)
    ↓
Atlantean UI / Terminal / Headless CI
```

---

## The Reducer Rule (strongest invariant)

```rust
pub trait Reducer<E> {
    fn apply(state: &RealmState, event: E) -> Result<RealmState>;
}
```

**No crate may mutate state structs directly.**
Immutable snapshots. Structural sharing. Copy-on-write. Zero mutable globals.
Without this: replay correctness depends on "discipline" — that breaks at scale.

---

## 22 Architecture Gaps (v1+v2+v3)

### Critical (must fix in Batch 0)

| # | Gap | Fix | Batch |
|---|---|---|---|
| 1 | Multiple state mutators — no canonical reducer | `bkg-state` Reducer<E> | 0 |
| 2 | No Universal Realm ABI — crates speak different serialization | `bkg-abi` + `AbiEnvelope<T>` | 0 |
| 3 | No deterministic clocks — `SystemTime::now()` breaks replay | `bkg-clock` vector clocks | 0 |
| 4 | Events untyped (`serde_json::Value`) — no compile-time replay validation | `DomainEvent<T>` | 0 |
| 5 | No EventSchemaRegistry — replay migrations impossible | `bkg-schema` | 0 |
| 6 | Kernel too passive — no causality judge | `kernel/arbitrator.rs` | 0 |
| 7 | No ABI version negotiation — old nodes unreadable | `AbiVersion + AbiEnvelope<T>` | 0 |
| 8 | ECS in Batch 4 too late — physics/compiler/world need it | `bkg-ecs` → Batch 1.5 | 1.5 |
| 9 | No projection cache — UI reads ledger directly (O(n) per frame) | `bkg-projection` | 1.5 |
| 10 | No canonical entity model — Task/Mission/Agent have no shared base | All become ECS entities | 1.5 |

### High (fix before Batch 3)

| # | Gap | Fix | Batch |
|---|---|---|---|
| 11 | Realm isolation "soft" — any realm could accept any event | `RealmDNA` | 1.5 |
| 12 | Capsule has no lifecycle SM | extend `bkg-capsule` | 1.5 |
| 13 | Workflow has no ExecutionGraph | `workflow/graph.rs` | 1 |
| 14 | No deterministic identity fabric — timeline forking uncontrollable | `bkg-identity` | 1.5 |
| 15 | Projection could become stale source-of-truth | checksumming + invalidation | 1.5 |
| 16 | No BQL — UI filters/telemetry/AI context extraction = chaos | `bkg-query` | 1 |

### Medium (fix before Batch 5)

| # | Gap | Fix | Batch |
|---|---|---|---|
| 17 | No causal GC — event log grows forever ("10 TB replay startup") | `bkg-gc` | 4 |
| 18 | No schema migration — old snapshots/replays break on changes | `bkg-migration` | 3 |
| 19 | No world snapshot — can't fork/restore full reality | `bkg-snapshot` | 3 |
| 20 | No entropy metrics — system health invisible | `bkg-entropy` | 4 |
| 21 | No deterministic execution simulator — can't test without real agents | `bkg-simulation` | 4 |
| 22 | Async determinism (tokio not fully replay-safe) | tick-driven executor | research |

---

## Complete Crate Map (58 target)

### ✅ DONE (24)

| Crate | Location |
|---|---|
| bkg-core | cognition/core |
| bkg-crypto | domains/katoptron/crypto |
| bkg-event | cognition/event |
| bkg-contracts | cognition/contracts |
| bkg-kernel | cognition/kernel |
| bkg-swd | domains/styx/provider |
| bkg-capsule | domains/arche/capsule |
| bkg-store | domains/arche/store |
| bkg-memory | domains/mnemos/memory |
| bkg-replay | domains/mnemos/replay |
| bkg-verifier | domains/katoptron/verifier |
| bkg-policy | domains/anamnesis/policy |
| bkg-runtime | domains/thalassa/runtime |
| bkg-orchestrator | domains/thalassa/orchestrator |
| bkg-exec | domains/thalassa/exec |
| bkg-tools | domains/styx/tools |
| bkg-inspector | reflection/inspector |
| bkg-providers | domains/thalassa/providers |
| bkg-telemetry | domains/katoptron/telemetry |
| bkg-agents | domains/thalassa/agents |
| bkg-session | domains/thalassa/session |
| bkg-acp | cognition/protocol |
| bkg-atlantean | reflection/ui/atlantean |
| bkg-cli | threshold/cli |

### 📋 PLANNED (+34)

#### Batch 0 — Architecture Foundation
| Crate | Location |
|---|---|
| bkg-state | cognition/state |
| bkg-abi | cognition/abi |
| bkg-clock | cognition/clock |
| bkg-schema | cognition/schema |

#### Batch 1 — Core Application
| Crate | Location |
|---|---|
| bkg-project | cognition/project |
| bkg-workflow | cognition/workflow |
| bkg-query | cognition/query |
| bkg-task | domains/thalassa/task |
| bkg-mission | domains/thalassa/mission |
| bkg-scheduler | domains/thalassa/scheduler |
| bkg-lanes | domains/styx/lanes |

#### Batch 1.5 — ECS Foundation
| Crate | Location |
|---|---|
| bkg-ecs | domains/katoptron/ecs |
| bkg-projection | domains/katoptron/projection |
| bkg-identity | domains/speculum/identity |

#### Batch 2 — Security + Features
| Crate | Location |
|---|---|
| bkg-secrets | domains/katoptron/secrets |
| bkg-approval | domains/katoptron/approval |
| bkg-capabilities | domains/speculum/capabilities |
| bkg-eval | domains/katoptron/eval |
| bkg-chat | domains/thalassa/chat |
| bkg-github | domains/thalassa/github |
| bkg-plugins | domains/katoptron/plugins |

#### Batch 3 — Infrastructure
| Crate | Location |
|---|---|
| bkg-mesh | domains/arche/mesh |
| bkg-vm | domains/thalassa/vm |
| bkg-snapshot | domains/speculum/snapshot |
| bkg-migration | domains/speculum/migration |

#### Batch 4 — Advanced Systems
| Crate | Location |
|---|---|
| bkg-physics | domains/katoptron/physics |
| bkg-entropy | domains/katoptron/entropy |
| bkg-compiler | domains/katoptron/compiler |
| bkg-render | domains/katoptron/render |
| bkg-diff | domains/speculum/diff |
| bkg-recovery | domains/speculum/recovery |
| bkg-gc | domains/speculum/gc |
| bkg-lineage | domains/speculum/lineage |
| bkg-simulation | domains/thalassa/simulation |
| bkg-world | domains/katoptron/world |
| bkg-operator | domains/anamnesis/operator |

#### Batch 5 — Consensus + Final
| Crate | Location |
|---|---|
| bkg-consensus | domains/speculum/consensus |

---

## Key Design Contracts

### `bkg-state` Reducer contract
```rust
// The ONLY allowed state mutation path.
// No exceptions. No "fast paths". No shortcuts.
pub trait Reducer<E: EventPayload> {
    fn apply(state: &RealmState, event: DomainEvent<E>) -> Result<RealmState>;
    fn schema_id() -> EventSchemaId;
}
```

### `bkg-abi` AbiEnvelope contract
```rust
// Every cross-system message is wrapped.
// Enables version negotiation for mesh + plugin compatibility.
pub struct AbiEnvelope<T> {
    pub abi_version:  AbiVersion,
    pub payload_type: Symbol,
    pub payload_hash: Hash256,
    pub payload:      T,
}
```

### `bkg-clock` SequencedInstant contract
```rust
// NO SystemTime::now() anywhere in business logic.
// wall_nanos is display-only and never used for ordering.
pub struct SequencedInstant {
    pub realm_id:       RealmId,
    pub lamport:        u64,
    pub wall_nanos:     u64,  // display only — not for ordering
}
```

### `bkg-schema` EventSchema contract
```rust
pub struct EventSchema {
    pub id:                  EventSchemaId,
    pub version:             SchemaVersion,
    pub producer_realm:      RealmId,
    pub reducer:             ReducerId,
    pub projection_targets:  Vec<ProjectionId>,
    pub causal_requirements: Vec<EventSchemaId>,
    pub migration_strategy:  MigrationStrategy,
}
```

### `RealmDNA` contract
```rust
pub struct RealmDNA {
    pub allowed_events:             Vec<EventSchemaId>,
    pub allowed_components:         Vec<ComponentTypeId>,
    pub allowed_capabilities:       Vec<CapabilityId>,
    pub allowed_lanes:              Vec<LaneClass>,
    pub allowed_reducers:           Vec<ReducerId>,
    pub allowed_projection_targets: Vec<ProjectionId>,
    pub allowed_snapshot_scopes:    Vec<SnapshotScope>,
    pub allowed_tick_domains:       Vec<TickDomain>,
    pub allowed_physics_rules:      Vec<PhysicsRuleId>,
}
```

### `bkg-projection` Projection contract
```rust
// Projections are DISPOSABLE. Never the source of truth.
// If stale: rebuild from ledger via bkg-state Reducer.
pub trait Projection: Sized {
    fn rebuild(ledger: &dyn EventLedger) -> Result<Self>;
    fn checksum(&self) -> Hash256;
    fn is_stale(&self, current: Hash256) -> bool;
}
```

### `bkg-ecs` World contract
```rust
// Deterministic iteration order guaranteed.
// No random hashing. No HashMap without stable order.
// Generation IDs prevent use-after-free entity confusion.
pub trait WorldQuery {
    fn entities<C: Component>(&self) -> impl Iterator<Item=(Entity, &C)>;
    // ^ always yields in stable order (insertion order)
}
```

---

## Fusion Features Status

Full mapping in `FEATURES.md`. Quick summary:

| Group | Crates needed | Status |
|---|---|---|
| Task Lifecycle (9) | bkg-task | 📋 Batch 1 |
| Workflow (12) | bkg-workflow | 📋 Batch 1 |
| Git/Worktree (7) | bkg-task (capsules) | 📋 Batch 1 |
| Agent Management (10) | bkg-agents ✅ | ✅ DONE |
| Multi-Project (7) | bkg-project | 📋 Batch 1 |
| AI Models (12) | bkg-providers ✅ | ✅ DONE |
| Mission (10) | bkg-mission | 📋 Batch 1 |
| Research/Insights (8) | bkg-memory ✅ + bkg-simulation | 🔨 PARTIAL |
| Evaluations (6) | bkg-eval | 📋 Batch 2 |
| Chat (8) | bkg-chat | 📋 Batch 2 |
| Terminal/DevServer (5) | bkg-vm + bkg-atlantean | 📋 Batch 3 |
| Automation (7) | bkg-orchestrator ✅ | 🔨 PARTIAL |
| Approvals (5) | bkg-approval | 📋 Batch 2 |
| Secrets (7) | bkg-secrets | 📋 Batch 2 |
| Plugins (8) | bkg-plugins | 📋 Batch 2 |
| Dashboard UI (11) | bkg-atlantean ✅ + extend | 🔨 PARTIAL |
| CLI (17) | bkg-cli ✅ + extend | 🔨 PARTIAL |
| Multi-Node Mesh (10) | bkg-mesh | 📋 Batch 3 |
| Remote Access (7) | bkg-mesh + bkg-atlantean | 📋 Batch 3 |
| Docker (5) | bkg-vm | 📋 Batch 3 |
| GitHub (7) | bkg-github | 📋 Batch 2 |
| Persistence (7) | bkg-store ✅ + bkg-task | ✅ PARTIAL |
| Observability (8) | bkg-telemetry ✅ + bkg-entropy | 🔨 PARTIAL |
| Sandbox (6) | bkg-vm + bkg-capabilities | 📋 Batch 3 |
| Desktop/Mobile (8) | bkg-atlantean (PWA) | 📋 Batch 5 |
| Settings (5) | bkg-project | 📋 Batch 1 |
| Onboarding (5) | bkg-atlantean ✅ | ✅ DONE |
| Advanced (27) | various | 📋 Batch 4 |

---

*BKG v0.1.0 · DELPHOS v3 · Deterministic Ontology Engine*
*58 crates · 258+ Fusion features · 22 architecture gaps mapped*
*Single source of truth. One module, one location.*

---

# BKG — TASK.md v3
## Current Status · Full Roadmap · Known Issues

> Last updated: 2026-05 · v3 — deterministic ontology engine
> Status: 24 crates in git · 7 commits · all tests pass · awaiting approval for Batch 0

---

## Philosophical Shift (v2 → v3)

BKG has crossed a threshold. It is no longer "agent framework with task board". It is:

> **deterministic causal operating substrate with replayable world simulation**

The AI agents are *inhabitants* of this world. Not the core.

The architecture in v3 closes all remaining gaps identified in the analysis:
- Single canonical state reducer (`bkg-state`)
- Universal Realm ABI with version negotiation (`bkg-abi`)
- EventSchemaRegistry for replay-safe migrations (`bkg-schema`)
- Deterministic sparse archetype ECS — moved to Batch 1.5 (`bkg-ecs`)
- Projection cache layer — UI reads projections only (`bkg-projection`)
- Deterministic lineage + ancestry IDs (`bkg-identity`)
- Causal garbage collection (`bkg-gc`) — prevents "10 TB replay startup"
- BQL query language separate from World Model (`bkg-query`)
- Render backend abstraction (`bkg-render`)
- Deterministic execution simulator (`bkg-simulation`)
- Timeline ancestry graph (`bkg-lineage`)
- Mesh consensus (Batch 5, research) (`bkg-consensus`)

---

## Git History

### ✅ 6578342 — BKG v0.1.0 (18 crates)
Original foundation: bkg-core, bkg-crypto, bkg-event, bkg-contracts, bkg-kernel,
bkg-swd, bkg-capsule, bkg-store, bkg-memory, bkg-replay, bkg-verifier, bkg-policy,
bkg-runtime, bkg-orchestrator, bkg-exec, bkg-tools, bkg-inspector, bkg-cli, bkg-testing

### ✅ 65601fa — pi-free integration
bkg-providers (13 providers, free detection, toggles, CI scores, quota)
bkg-telemetry (model call tracking, quota monitor)
bkg-verifier extended: PermissionEnforcer (ReadOnly/WorkspaceWrite/DangerFullAccess)
bkg-cli extended: `bkg providers`, `bkg chat`, `bkg agent`

### ✅ 3d5a333 — bkg-atlantean dashboard
Cyberpunk/Atlantis UI. Private/Cloud mode switch. WebLLM + Ollama tunnel.
All provider + user + admin API endpoints. Onboarding wizard.

### ✅ 63c6ab2 — bkg-agents (sandbox-agent port)
7 agents, AgentMode (incl. BkgSupervised), credential fallback chain, live status probe.

### ✅ 83f7db8 — bkg-session (sandbox-agent port)
UniversalEvent (10 types), UniversalMessage (8 parts), BkgSession broadcast+replay, SSE.

### ✅ c0c8eed — bkg-acp (sandbox-agent port)
JSON-RPC 2.0, 24 ACP methods, AgentBridge stdout→Universal, InferenceProxy.

### ✅ 9155c47 → ba5f667 — Inspector + Agents UI
/agents/* endpoints, /sessions/* CRUD + SSE, Agents tab, Inspector tab.

### ✅ 8e43bd4 — docs v1 (task.md spec)
### ✅ d2c37ff — docs v2 (README + FEATURES + TASK)
### 📋 CURRENT — docs v3 (this commit)

---

## Current State

```
Crates:    24 implemented
Files:     130+ Rust source files
Tests:     ~155 unit tests, all passing
Clippy:    0 errors with -D warnings
```

---

## Roadmap

### Batch 0 — Architecture (MUST COME FIRST)
*Without these, application features build on wrong foundations.*

| Crate/Module | Location | Status | Notes |
|---|---|---|---|
| `bkg-state` | `cognition/state` | 📋 TODO | RealmStateMachine, Reducer<E>, immutable snapshots |
| `bkg-abi` | `cognition/abi` | 📋 TODO | Universal ABI + version negotiation, AbiEnvelope<T> |
| `bkg-clock` | `cognition/clock` | 📋 TODO | Vector clocks, SequencedInstant, no SystemTime::now() |
| `bkg-schema` | `cognition/schema` | 📋 TODO | EventSchemaRegistry, migration strategies |
| `DomainEvent<T>` | extend `bkg-event` | 📋 TODO | Typed events with schema_id + causal_parent |
| kernel arbitrator | extend `bkg-kernel` | 📋 TODO | Causality judge, replay paradox prevention |
| workflow ExecutionGraph | in `bkg-workflow` | 📋 TODO | Loops, retries, parallel waves, conditionals |

**Estimated: 7 PRs, each pushed immediately after tests pass**

---

### Batch 1 — Core Application

| Crate | Status | Priority |
|---|---|---|
| `bkg-task` | 📋 TODO | 🔴 CRITICAL |
| `bkg-project` | 📋 TODO | 🔴 CRITICAL |
| `bkg-workflow` | 📋 TODO | 🔴 HIGH |
| `bkg-scheduler` | 📋 TODO | 🔴 HIGH |
| `bkg-mission` | 📋 TODO | 🟠 HIGH |
| `bkg-query` (BQL engine) | 📋 TODO | 🟠 HIGH |
| `bkg-lanes` | 📋 TODO | 🟠 HIGH |

---

### Batch 1.5 — ECS Foundation (MOVED UP — physics/compiler/world need this)

| Crate | Status | Priority |
|---|---|---|
| `bkg-ecs` (deterministic sparse archetype) | 📋 TODO | 🔴 CRITICAL |
| `bkg-projection` | 📋 TODO | 🔴 CRITICAL |
| `bkg-identity` | 📋 TODO | 🟠 HIGH |
| capsule lifecycle SM | extend `bkg-capsule` | 🟠 HIGH |
| RealmDNA | `cognition/realm-dna` | 🟠 HIGH |

---

### Batch 2 — Security + Features

| Crate | Status | Priority |
|---|---|---|
| `bkg-secrets` | 📋 TODO | 🟠 HIGH |
| `bkg-approval` | 📋 TODO | 🟠 HIGH |
| `bkg-capabilities` | 📋 TODO | 🟠 HIGH |
| `bkg-eval` | 📋 TODO | 🟡 MEDIUM |
| `bkg-chat` | 📋 TODO | 🟡 MEDIUM |
| `bkg-github` | 📋 TODO | 🟡 MEDIUM |
| `bkg-plugins` | 📋 TODO | 🟡 MEDIUM |

---

### Batch 3 — Infrastructure

| Crate | Status | Priority |
|---|---|---|
| `bkg-mesh` | 📋 TODO | 🟠 HIGH |
| `bkg-vm` | 📋 TODO | 🟠 HIGH |
| `bkg-snapshot` | 📋 TODO | 🟡 MEDIUM |
| `bkg-migration` | 📋 TODO | 🟡 MEDIUM |

---

### Batch 4 — Advanced Systems

| Crate | Status | Priority |
|---|---|---|
| `bkg-physics` | 📋 TODO | 🟠 HIGH |
| `bkg-entropy` | 📋 TODO | 🟡 MEDIUM |
| `bkg-compiler` | 📋 TODO | 🟠 HIGH |
| `bkg-render` | 📋 TODO | 🟡 MEDIUM |
| `bkg-diff` | 📋 TODO | 🟡 MEDIUM |
| `bkg-recovery` | 📋 TODO | 🟡 MEDIUM |
| `bkg-gc` | 📋 TODO | 🟠 HIGH |
| `bkg-lineage` | 📋 TODO | 🟡 MEDIUM |
| `bkg-simulation` | 📋 TODO | 🟡 MEDIUM |
| `bkg-world` (ULTIMATE) | 📋 TODO | 🔴 CRITICAL |
| `bkg-operator` | 📋 TODO | 🟡 MEDIUM |

---

### Batch 5 — Consensus + Final UI

| Task | Status | Priority |
|---|---|---|
| `bkg-consensus` | 📋 TODO | 🟡 MEDIUM |
| Atlantean: Kanban + Task detail + Mission + Physics | 📋 TODO | 🟠 HIGH |
| CLI: all Fusion commands | 📋 TODO | 🟠 HIGH |
| Terminal ratatui backend | 📋 TODO | 🟡 MEDIUM |
| Headless CI backend | 📋 TODO | 🟡 MEDIUM |

---

## Known Issues

### Architecture (must fix in Batch 0)
- [ ] **Multiple state mutators** — bkg-runtime, bkg-workflow, bkg-scheduler all can mutate state. `bkg-state` Reducer must unify them.
- [ ] **Untyped events** — `serde_json::Value` payload breaks compile-time replay validation. `DomainEvent<T>` needed.
- [ ] **`SystemTime::now()` in session.rs and tracker.rs** — violates replay invariant. Replace with `bkg-clock`.
- [ ] **No ABI versioning** — mesh nodes from different versions will be incompatible. `bkg-abi` + `AbiEnvelope<T>`.
- [ ] **ECS in Batch 4 was too late** — physics, compiler, and world all need it. Moved to Batch 1.5.
- [ ] **Projection missing** — Atlantean reads from state directly. Needs `bkg-projection` materialized read models.

### Implementation
- [ ] **bkg-workflow is an empty stub** — `cognition/workflow/src/lib.rs` has no implementation.
- [ ] **bkg-acp AgentBridge doesn't spawn real processes** — `translate_stdout()` works, but spawn loop not wired to real agents.
- [ ] **bkg-session in-memory only** — sessions don't survive server restart. Needs `bkg-store` persistence.
- [ ] **bkg-telemetry Cargo.toml lost on workspace reset** — tracked in git now but verify.
- [ ] **bkg-providers auto-refresh on startup** — `bkg providers list` shows 0 models until `refresh` is run.

### Dashboard
- [ ] **Inspector SSE drops on restart** — needs client-side reconnect logic with backoff.
- [ ] **Admin key masking edge case** — partial unmasking in some browser states.
- [ ] **No Kanban board yet** — biggest missing UI feature.

---

## Test Coverage

| Crate | Tests | Pass |
|---|---|---|
| bkg-core | 7 | ✅ |
| bkg-crypto | 4 | ✅ |
| bkg-event | 9 | ✅ |
| bkg-contracts | 7 | ✅ |
| bkg-kernel | 9 | ✅ |
| bkg-swd | 9 | ✅ |
| bkg-capsule | 12 | ✅ |
| bkg-store | 7 | ✅ |
| bkg-memory | 2 | ✅ |
| bkg-replay | 7 | ✅ |
| bkg-verifier | 13 | ✅ |
| bkg-policy | 4 | ✅ |
| bkg-runtime | 7 | ✅ |
| bkg-providers | 17 | ✅ |
| bkg-telemetry | 2 | ✅ |
| bkg-agents | 10 | ✅ |
| bkg-session | 14 | ✅ |
| bkg-acp | 14 | ✅ |
| rest | stubs | — |
| **Total** | **~155** | **all pass** |

---

## Architectural Decisions Log

### 2026-04: DELPHOS over Fusion
Fusion = application layer. BKG = the deterministic substrate beneath it.
All 258+ Fusion features map onto DELPHOS realms. No Fusion code imported as-is.

### 2026-04: pi-free → bkg-providers
Full Rust rewrite. 13 providers. Extended with tier metadata, signup_url, BkgSupervised mode.

### 2026-04: sandbox-agent → bkg-agents + bkg-session + bkg-acp
Portable to BKG structure. `_bkg/` replaces `_sandboxagent/`. BkgSupervised extends AgentMode.

### 2026-05: ECS to Batch 1.5 (was Batch 4)
Physics, compiler, and world need ECS as foundation. Standard ECS libs break replay determinism.
Need: deterministic sparse archetype ECS with stable iteration order + replay-safe allocation.

### 2026-05: bkg-state is Batch 0 Critical
Without canonical reducer: multiple crates diverge. Replay correctness depends on "discipline" (fragile).
`apply(state: &RealmState, event: E) -> Result<RealmState>` = the ONLY state mutator.

### 2026-05: bkg-world is the True Kernel
Once Ledger → State → ECS → World Graph is implemented, bkg-world becomes the integration layer.
Physics, UI, AI planning, telemetry, mission orchestration = different projections of same world.

### 2026-05: Causal GC is mandatory
Without bkg-gc: event log grows forever → "10 TB replay startup" at scale.
Strategy: causal compaction → snapshot sealing → timeline freezing → projection pruning.

### 2026-05: Async determinism is a research problem
Full `same seed + same ledger = same output` requires deterministic executor + tick-driven async.
tokio alone is insufficient. Deferred to research track (Batch 5+).

---

## The Causal Data Flow

```
Events
    ↓ DomainEvent<T> (typed, signed, causal parent)
bkg-event (ledger, hash-chained, BLAKE3)
    ↓ apply() — bkg-state (ONLY state mutator)
RealmState (immutable, copy-on-write)
    ↓ RealmDNA validation
Validated State
    ↓ entity sync — bkg-ecs (deterministic sparse archetype)
World Graph — bkg-world (entities + relations + causality)
    ↓ physics — bkg-physics (n-body, deterministic)
Geometry
    ↓ compile — bkg-compiler (bytecode via bkg-abi)
Render Bytecode
    ↓ materializer — bkg-projection (indexed, checksummed)
Projection Cache
    ↓ queries — bkg-query (BQL engine)
Atlantean UI / Terminal / Headless
```

---

*BKG v0.1.0 · DELPHOS · Deterministic Ontology Engine*
*58 target crates · 258+ Fusion features · 22 architecture gaps resolved*
*Single source of truth. One module, one location.*
