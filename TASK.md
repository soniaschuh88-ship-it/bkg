# BKG — TASK.md
## Current Status, Roadmap, Known Issues

> Last updated: 2026-05
> Status: Foundation complete · Awaiting approval for Batch 0

---

## Current State

```
Git:      main @ 8e43bd4
Crates:   24 (all compiling, all tests pass)
Tests:    130+ Rust files, all clean
Clippy:   0 warnings with -D warnings
```

---

## Completed Work

### ✅ Commit 6578342 — BKG v0.1.0 Foundation (18 crates)
The original BKG workspace — deterministic execution system core.

**Crates:**
- `bkg-core` — typed IDs, Hash256, BkgError, ExecutionSeed, Capabilities
- `bkg-crypto` — BLAKE3, Ed25519, seed derivation
- `bkg-event` — Event, EventLedger (memory + file), LaneEvent
- `bkg-contracts` — CausalContract cross-realm messaging
- `bkg-kernel` — Genesis lock, RealmRouter, CausalContractValidator
- `bkg-swd` — SwdEngine (init → capture → commit → verify → archive)
- `bkg-capsule` — Capsule + CapsuleManager
- `bkg-store` — InMemoryStore + SledStore
- `bkg-memory` — MemoryGraph (impact × recurrence × depth)
- `bkg-replay` — ReplayEngine, DivergenceDetector, BranchReport
- `bkg-verifier` — hash-chain + capsule integrity + drift detection
- `bkg-policy` — PolicyEngine + built-in event policies
- `bkg-runtime` — AgentRuntime (Telum sandbox)
- `bkg-orchestrator` — TaskGraph (DAG), EventBus, Scheduler
- `bkg-tools` — ledger_summary, dump_realm
- `bkg-inspector` — realm name registry
- `bkg-cli` — `bkg` binary: init, run, verify, replay, status, isolate
- `bkg-testing` — shared test fixtures

### ✅ Commit 65601fa — pi-free Integration
**bkg-providers** (pi-free fully refactored to Rust):
- 13 providers: Ollama, NVIDIA, OpenRouter, SambaNova, LLM7, Kilo, Cline, ZenMux, CrofAI, Codestral, DeepInfra, Together, Novita
- Tier metadata + signup_url for onboarding wizard
- FreeModelDetector: Route A (cost-based) + Route B (name-based)
- ProviderRegistry with per-provider toggle (persisted)
- CI score enhancer, fetch helpers

**bkg-telemetry** (pi-free telemetry refactored):
- ModelCallRecord, ModelStats, ModelTracker + JSON persistence
- QuotaMonitor + persistence

**bkg-verifier** extension:
- PermissionEnforcer: PermissionMode, EnforcementResult, PermissionRequest
- 8 unit tests

**bkg-cli** extension:
- `bkg providers` command: list/models/toggle/refresh/telemetry/quota
- `bkg chat` with /slash commands and Anthropic/OpenAI/Ollama auto-detection
- `bkg agent` command: list/spawn/show

### ✅ Commit 3d5a333 — bkg-atlantean Dashboard
- Cyberpunk/Atlantis design (Orbitron + Exo 2, animated particle grid)
- Private/Cloud mode switch (WebLLM + Ollama tunnel / 13 providers)
- All API endpoints: /api/mode, /api/models, /providers/*, /user/*, /admin/*, /tunnel/ollama/*
- Onboarding wizard (3-step, bkg_XXXXXXX key generation)
- Admin dashboard (global provider keys, default model, free-only toggle)
- User dashboard (provider keys by tier, inline edit)

### ✅ Commit 63c6ab2 — bkg-agents (sandbox-agent port)
- 7 agents: Claude, Codex, OpenCode, Amp, Pi, Cursor, Mock
- AgentMode: Default | Bypass | PlanMode | **BkgSupervised** (new)
- AgentInfo per agent (streaming, permissions, file_ops, images)
- AgentCredentials fallback chain
- AgentStatus: live binary probe
- InstallOptions / install recipes
- 10 unit tests

### ✅ Commit 83f7db8 — bkg-session (sandbox-agent port)
- UniversalEvent: 10 event types
- UniversalMessage: Parsed/Unparsed + 8 part types
- BkgSession: broadcast streaming + offset-based replay
- SessionManager, PermissionStrategy, SseEvent
- 14 unit tests

### ✅ Commit c0c8eed — bkg-acp (sandbox-agent port)
- JSON-RPC 2.0 types: RpcRequest, RpcResponse, RpcError
- 24 ACP methods (session/*, agent/*, process/*, file/*, _bkg/*)
- AgentBridge: stdout → UniversalEventData translation (Claude, Codex, OpenCode, Mock)
- InferenceProxy: provider fallback chain
- 14 unit tests

### ✅ Commit 9155c47 → ba5f667 — Atlantean: Inspector + Agents
- /agents/list, /agents/:id/status, /agents/:id/credentials
- /sessions CRUD + /sessions/:id/send + /sessions/:id/stream (SSE)
- Agents tab: 7 cards with status, credentials, mode badges
- Inspector tab: session browser + live SSE event viewer + offset replay

---

## Planned Roadmap

### 📋 Batch 0 — Critical Architecture (IMMEDIATE)
*These must come before any application features. They are the foundation.*

| Task | Crate | Priority | Notes |
|---|---|---|---|
| RealmStateMachine | `bkg-state` | 🔴 CRITICAL | Single canonical reducer — no more scattered mutations |
| Universal Realm ABI | `bkg-abi` | 🔴 CRITICAL | Standardize all inter-system communication |
| Realm Clock | `bkg-clock` | 🔴 CRITICAL | Vector clocks, causal ordering, no SystemTime::now() |
| Kernel Arbitrator | extend `bkg-kernel` | 🔴 CRITICAL | Prevents causality corruption + replay paradoxes |
| Workflow ExecutionGraph | extend `bkg-workflow` | 🟠 HIGH | Loops, retries, parallel waves, conditional transitions |
| DomainEvent\<T\> typing | extend `bkg-event` | 🟠 HIGH | Typed events for compile-time replay validation |

### 📋 Batch 1 — Core Application Features

| Task | Crate | Priority | Notes |
|---|---|---|---|
| Task capsule system | `bkg-task` | 🔴 CRITICAL | `.bkg/tasks/{id}/` layout, lifecycle, DAG deps |
| Project registry | `bkg-project` | 🔴 CRITICAL | `~/.bkg/bkg-central.db`, settings, isolation |
| Workflow gates | `bkg-workflow` | 🟠 HIGH | Plan→Review→Execute + verdicts + waves |
| Deterministic scheduler | `bkg-scheduler` | 🟠 HIGH | DAG, priority queue, overlap gating, leases |
| Mission hierarchy | `bkg-mission` | 🟠 HIGH | Mission→Milestone→Slice→Feature→Task |
| Realm Bus IPC | `bkg-lanes` | 🟠 HIGH | Deterministic inter-realm transport |

### 📋 Batch 1.5 — ECS Foundation (moved up from Batch 4)
*ECS must come before Physics, Compiler, and World Model*

| Task | Crate | Priority | Notes |
|---|---|---|---|
| Entity Component System | `bkg-ecs` | 🔴 CRITICAL | All DELPHOS entities become ECS entities |
| Projection Cache | `bkg-projection` | 🟠 HIGH | UI reads projections only, not ledger directly |
| Realm DNA | `cognition/realm-dna` | 🟠 HIGH | Per-realm allowed events/mutations/capabilities |
| Capsule lifecycle SM | extend `bkg-capsule` | 🟠 HIGH | Created/Mounted/Active/Frozen/Forked/Archived |

### 📋 Batch 2 — Security + Features

| Task | Crate | Priority | Notes |
|---|---|---|---|
| Encrypted secrets | `bkg-secrets` | 🟠 HIGH | AES-256-GCM, OS keychain, scope policies |
| Approval gates | `bkg-approval` | 🟠 HIGH | Audit trail, dedup, action classification |
| Realm capabilities | `bkg-capabilities` | 🟠 HIGH | Signed scopes, revocable grants |
| Task evaluations | `bkg-eval` | 🟡 MEDIUM | Scorecards, evidence, scheduled batches |
| Chat rooms | `bkg-chat` | 🟡 MEDIUM | Rooms, mailbox, SSE streaming |
| GitHub integration | `bkg-github` | 🟡 MEDIUM | Issue import, PR creation, OAuth |
| Plugin system | `bkg-plugins` | 🟡 MEDIUM | YAML manifest, UI slots, prompt contributions |

### 📋 Batch 3 — Infrastructure

| Task | Crate | Priority | Notes |
|---|---|---|---|
| Multi-node mesh | `bkg-mesh` | 🟠 HIGH | mDNS discovery, lease management, write queue |
| Tool sandbox VM | `bkg-vm` | 🟠 HIGH | Syscall layer, VFS mounts, resource limits, rollback |
| Global snapshots | `bkg-snapshot` | 🟡 MEDIUM | fork/export/restore full world state |

### 📋 Batch 4 — Advanced Systems

| Task | Crate | Priority | Notes |
|---|---|---|---|
| DAG physics engine | `bkg-physics` | 🟠 HIGH | Node mass, edge tension, entropy, n-body layout |
| Katoptron UI compiler | `bkg-compiler` | 🟠 HIGH | Ledger → AST → Geometry → Bytecode |
| Reality diff engine | `bkg-diff` | 🟡 MEDIUM | State/graph/capsule/timeline diffs |
| Realm recovery | `bkg-recovery` | 🟡 MEDIUM | Crash reconstruction, partial replay repair |
| Causal world model | `bkg-world` | 🔴 ULTIMATE | The final integration: World Graph + BQL |
| Operator consciousness | `bkg-operator` | 🟡 MEDIUM | Intent tracking, adaptive orchestration |
| Telemetry physics | extend `bkg-telemetry` | 🟡 MEDIUM | entropy, pressure, heat, stability |

### 📋 Batch 5 — UI + CLI

| Task | Component | Priority | Notes |
|---|---|---|---|
| Kanban board | `bkg-atlantean` | 🟠 HIGH | Drag-drop task columns |
| Task detail modal | `bkg-atlantean` | 🟠 HIGH | PROMPT.md, logs, diffs, workflow steps |
| Mission browser | `bkg-atlantean` | 🟡 MEDIUM | Hierarchy + autopilot controls |
| Physics DAG view | `bkg-atlantean` | 🟡 MEDIUM | Live physics simulation |
| Reality diff page | `bkg-atlantean` | 🟡 MEDIUM | Timeline divergence visualization |
| All Fusion CLI commands | `bkg-cli` | 🟠 HIGH | task/mission/project/workflow/secrets... |
| Terminal ratatui backend | `bkg-compiler/backends` | 🟡 MEDIUM | ASCII physics terminal |

---

## Known Issues

### Architecture
- [ ] **`bkg-event` lacks `DomainEvent<T>`** — events are currently untyped `serde_json::Value` payloads. Typed events needed for compile-time replay validation.
- [ ] **No canonical state reducer** — multiple crates can mutate state independently without going through a single `apply(state, event)` path. `bkg-state` must fix this.
- [ ] **`SystemTime::now()` used in some crates** — violates replay invariant. Must be replaced with `bkg-clock` `SequencedInstant` everywhere.
- [ ] **bkg-telemetry Cargo.toml gets lost on workspace reset** — not tracked in git correctly. Investigate.
- [ ] **ECS in Batch 4 is too late** — Physics, Compiler, and World Model all need ECS as a foundation. Must move to Batch 1.5.
- [ ] **No Universal Realm ABI** — every crate speaks slightly different serialization. `bkg-abi` needed before mesh + plugin work.

### Implementation
- [ ] **bkg-atlantean relative paths for bkg-agents/bkg-session** — fixed in last commit but needs regression test.
- [ ] **bkg-workflow is a stub** — the crate exists but has no implementation yet (empty lib.rs stub).
- [ ] **bkg-acp AgentBridge doesn't actually spawn processes** — `translate_stdout()` works, but the process spawn loop isn't connected to real agent binaries yet.
- [ ] **bkg-session SessionManager is in-memory only** — sessions don't survive server restarts. Need persistence via bkg-store.
- [ ] **bkg-providers has no `async-trait` in workspace.dependencies** — uses inline `async-trait = "0.1"` in some crates. Should be workspace dep.

### Dashboard
- [ ] **`bkg providers list` shows 0 models** — models are only cached after `bkg providers refresh`. Need auto-refresh on startup.
- [ ] **Inspector SSE connection drops on server restart** — client needs reconnect logic.
- [ ] **Admin page doesn't mask existing keys on load** — security issue (shows unmasked keys in some edge cases).

---

## Test Coverage

| Crate | Tests | Status |
|---|---|---|
| `bkg-core` | 7 | ✅ |
| `bkg-crypto` | 4 | ✅ |
| `bkg-event` | 9 | ✅ |
| `bkg-contracts` | 7 | ✅ |
| `bkg-kernel` | 9 | ✅ |
| `bkg-swd` | 9 | ✅ |
| `bkg-capsule` | 12 | ✅ |
| `bkg-store` | 7 | ✅ |
| `bkg-memory` | 2 | ✅ |
| `bkg-replay` | 7 | ✅ |
| `bkg-verifier` | 5+8 | ✅ |
| `bkg-policy` | 4 | ✅ |
| `bkg-runtime` | 7 | ✅ |
| `bkg-providers` | 17 | ✅ |
| `bkg-telemetry` | 2 | ✅ |
| `bkg-agents` | 10 | ✅ |
| `bkg-session` | 14 | ✅ |
| `bkg-acp` | 14 | ✅ |
| All others | 0 | stubs |
| **Total** | **~155** | **all pass** |

---

## Architecture Decisions Log

### 2026-04: DELPHOS over Fusion
- Decided to treat Fusion as an application layer, not the system core
- BKG's event-sourced architecture provides stronger guarantees than Fusion's SQLite-first design
- All Fusion features are mapped onto DELPHOS realms, not imported as-is

### 2026-04: pi-free → bkg-providers
- Ported from TypeScript to Rust for type safety + deterministic builds
- Extended with `tier` metadata and `signup_url` for onboarding wizard
- Added `BkgSupervised` agent mode (enforces Plan→Review→Execute gates)

### 2026-04: sandbox-agent → bkg-agents + bkg-session + bkg-acp
- Ported from Rust (upstream) to BKG's DELPHOS structure
- `AgentBridge.translate_stdout()` normalizes 3+ agent event formats to `UniversalEventData`
- `_bkg/` namespace replaces `_sandboxagent/` namespace in ACP methods

### 2026-05: ECS must be Batch 1.5 (not Batch 4)
- Physics simulation, UI compiler, and World Model all require entities as foundation
- Moving `bkg-ecs` earlier makes all subsequent batches cleaner
- Everything becomes an entity: Task, Mission, Agent, Session, Approval, Provider, MeshNode

### 2026-05: bkg-state is Batch 0 Critical
- Without a canonical reducer, multiple crates can produce divergent state
- `apply(state, event) -> Result<RealmState>` must be the ONLY state mutator
- Projection cache depends on this for correctness

---

## Deterministic OS Vision

When complete, DELPHOS will implement:

```
Not: "AI coding tool"
But: "deterministic causal OS for synthetic execution worlds"
```

The full data flow:

```
Event Ledger (bkg-event)
    ↓ apply() — bkg-state
Realm State
    ↓ entity creation — bkg-ecs
World Graph (bkg-world)
    ↓ forces — bkg-physics
Geometry
    ↓ compile — bkg-compiler
Render Bytecode
    ↓ projection — bkg-projection
Atlantean UI / Terminal
```

Every step is: deterministic, replayable, cryptographically verifiable.

---

*BKG v0.1.0 · DELPHOS architecture*
*Single source of truth. One module, one location.*
