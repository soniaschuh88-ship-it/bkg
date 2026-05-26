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
