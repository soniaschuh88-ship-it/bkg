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
