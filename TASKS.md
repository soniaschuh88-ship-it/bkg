# TASKS.md — Active Work + Roadmap

> Current sprint + full implementation queue.
> See `docs/PROGRESS.md` for completed work history.

---

## IN PROGRESS

### Batch 0 — Architecture Foundation
*Must complete before any application crates. These are the non-negotiable substrate.*

- [x] **bkg-state** `cognition/state` — `Reducer<E>` trait, immutable `RealmState`, projection layer, reconciliation
- [x] **bkg-abi** `cognition/abi` — `AbiEnvelope<T>`, version negotiation, typed serialization contracts
- [x] **bkg-clock** `cognition/clock` — `SequencedInstant`, `VectorClock`, no `SystemTime::now()`
- [x] **bkg-schema** `cognition/schema` — `EventSchemaRegistry`, migration strategies, schema versioning
- [x] **DomainEvent<T>** extend `bkg-event` — typed events, `schema_id`, `causal_parent`
- [x] **kernel/arbitrator** extend `bkg-kernel` — causality judge, replay paradox prevention
- [ ] **workflow ExecutionGraph** in `bkg-workflow` impl — loops, retries, parallel waves

---

## QUEUE

### Batch 1 — Core Application

- [x] `bkg-project` `cognition/project` — project registry, settings, 5 model lanes
- [x] `bkg-workflow` `cognition/workflow` — Plan→Review→Execute, verdicts, wave execution
- [ ] `bkg-query` `cognition/query` — BQL engine: `SELECT tasks WHERE status = "blocked" ORDER BY entropy`
- [ ] `bkg-task` `domains/thalassa/task` — task capsules `.bkg/tasks/{id}/`, lifecycle, DAG deps
- [x] `bkg-mission` `domains/thalassa/mission` — Mission→Milestone→Slice→Feature→Task, autopilot
- [x] `bkg-scheduler` `domains/thalassa/scheduler` — deterministic DAG, priority queue, overlap gating
- [ ] `bkg-lanes` `domains/styx/lanes` — Realm Bus IPC, priority lanes, backpressure

### Batch 1.5 — ECS Foundation

- [ ] `bkg-ecs` `domains/katoptron/ecs` — **deterministic sparse archetype ECS** (stable iteration order, replay-safe allocation, generation IDs)
- [ ] `bkg-projection` `domains/katoptron/projection` — `ProjectionCache`, materializer, invalidation, realtime subscriptions
- [ ] `bkg-identity` `domains/speculum/identity` — `DeterministicId::derive(seed, lineage, realm)`
- [ ] capsule lifecycle SM — extend `bkg-capsule` — Created/Mounted/Active/Frozen/Forked/Archived/Corrupted/Recovered
- [ ] RealmDNA — `cognition/realm-dna` — allowed events/components/capabilities/lanes/reducers per realm

### Batch 2 — Security + Features

- [ ] `bkg-secrets` `domains/katoptron/secrets` — AES-256-GCM, OS keychain, scopes, policies
- [ ] `bkg-approval` `domains/katoptron/approval` — gates, immutable audit trail, action classification
- [ ] `bkg-capabilities` `domains/speculum/capabilities` — realm-scoped permissions, signed scopes, revocation
- [ ] `bkg-eval` `domains/katoptron/eval` — scorecards, evidence, scheduled batches
- [ ] `bkg-chat` `domains/thalassa/chat` — rooms, mailbox, SSE streaming, mention routing
- [ ] `bkg-github` `domains/thalassa/github` — issue import, PR creation, OAuth, webhooks
- [ ] `bkg-plugins` `domains/katoptron/plugins` — YAML manifest, UI slots, prompt contributions

### Batch 3 — Infrastructure

- [ ] `bkg-mesh` `domains/arche/mesh` — multi-node replication, mDNS, lease management, write queue
- [ ] `bkg-vm` `domains/thalassa/vm` — tool sandbox VM, syscall layer, VFS mounts, resource limits
- [ ] `bkg-snapshot` `domains/speculum/snapshot` — `RealitySnapshot`, fork/export/restore
- [ ] `bkg-migration` `domains/speculum/migration` — replay-safe schema migrations

### Batch 4 — Advanced Systems

- [ ] `bkg-physics` `domains/katoptron/physics` — node mass, edge tension, entropy, n-body layout
- [ ] `bkg-entropy` `domains/katoptron/entropy` — pressure, heat, stability, drift metrics
- [ ] `bkg-compiler` `domains/katoptron/compiler` — `UiAst → Geometry → Bytecode`
- [ ] `bkg-render` `domains/katoptron/render` — ratatui / ANSI / WebGPU / headless backends
- [ ] `bkg-diff` `domains/speculum/diff` — state/graph/capsule/timeline diff
- [ ] `bkg-recovery` `domains/speculum/recovery` — crash reconstruction, partial replay repair
- [ ] `bkg-gc` `domains/speculum/gc` — causal compaction, snapshot sealing, projection pruning
- [ ] `bkg-lineage` `domains/speculum/lineage` — timeline ancestry graph
- [ ] `bkg-simulation` `domains/thalassa/simulation` — deterministic execution simulator
- [ ] `bkg-world` `domains/katoptron/world` — Causal World Model (the true kernel)
- [ ] `bkg-operator` `domains/anamnesis/operator` — operator consciousness, intent tracking

### Batch 5 — Consensus + UI + CLI

- [ ] `bkg-consensus` `domains/speculum/consensus` — Raft-inspired mesh arbitration
- [ ] Atlantean: Kanban board, Task detail modal, Mission browser, Physics DAG view, Reality diff page
- [ ] CLI: `bkg task`, `bkg mission`, `bkg project`, `bkg workflow`, `bkg secret`, `bkg eval`, `bkg mesh`, `bkg snapshot`
- [ ] Terminal ratatui backend (`bkg-render/ratatui`)
- [ ] Headless CI backend (`bkg-render/headless`)

---

## ENHANCEMENTS (queued)

### bkg-event (extend existing)
- [ ] `DomainEvent<T>` — typed generic wrapper replacing raw `serde_json::Value` payload
- [ ] `EventSchemaId` + `causal_parent: Option<EventId>` on every event
- [ ] Forward compatibility: unknown event types deserialized to `Unknown { raw }` not panic

### bkg-kernel (extend existing)
- [ ] `Arbitrator` — prevents: concurrent causality corruption, invalid realm transitions, duplicate tick chains, cyclic approvals, replay paradoxes
- [ ] Kernel becomes: BIOS + Hypervisor + Causality Judge

### bkg-capsule (extend existing)
- [ ] Lifecycle state machine: `Created → Mounted → Active → Frozen → Forked → Archived → Corrupted → Recovered`
- [ ] `CapsuleId` lineage tracking (parent → child → fork)

### bkg-verifier (extend existing)
- [ ] Integrate `bkg-diff` output into verification reports
- [ ] `DriftEvent` emission on hash mismatch (feeds into `bkg-state` reconciliation)

### bkg-telemetry (extend existing)
- [ ] Telemetry physics: entropy, pressure, heat, stability as physical system properties
- [ ] Integration with `bkg-entropy` once available

### bkg-providers (extend existing)
- [ ] Model ABI layer (`providers/abi/`): request/response normalization via `bkg-abi`
- [ ] Dynamic provider refresh on startup (not just on `bkg providers refresh`)

### bkg-session (extend existing)
- [ ] Persist sessions via `bkg-store` (survive server restart)
- [ ] Replace `SystemTime::now()` with `bkg-clock` `SequencedInstant`

### bkg-atlantean (extend existing)
- [ ] Kanban board page with drag-drop columns
- [ ] Task detail page (PROMPT.md viewer, logs, diffs, workflow steps)
- [ ] Mission browser with autopilot controls
- [ ] Physics DAG view (live simulation)
- [ ] Reality diff page (timeline divergence)
- [ ] Chat rooms page (multi-agent group chat)
- [ ] Secrets management page
- [ ] Approvals queue page
- [ ] Mesh node health page
- [ ] Plugin discovery + install page
- [ ] SSE reconnect logic with exponential backoff
- [ ] PWA manifest (desktop/mobile install)

### bkg-cli (extend existing)
- [ ] `bkg task create/plan/show/logs/steer/archive`
- [ ] `bkg mission create/show/activate-slice`
- [ ] `bkg project add/list/default`
- [ ] `bkg workflow approve/revise/rethink`
- [ ] `bkg secret set/get/list/delete`
- [ ] `bkg eval run/list`
- [ ] `bkg mesh status/nodes`
- [ ] `bkg snapshot create/fork/export/diff/restore`
- [ ] `bkg settings set/export`
- [ ] `bkg serve` — headless daemon mode

---

## KNOWN ISSUES

- [x] `bkg-workflow` is an empty stub — needs full implementation in Batch 0
- [ ] `bkg-acp` `AgentBridge` not wired to real agent processes
- [ ] `bkg-session` in-memory only — add `bkg-store` persistence
- [ ] `SystemTime::now()` in `session.rs`, `tracker.rs` — replace with `bkg-clock`
- [ ] Atlantean `bkg providers list` shows 0 models until explicit refresh
- [ ] Inspector SSE drops on server restart — add client reconnect
- [ ] Admin key masking edge case in browser

---

## RULES (non-negotiable)

1. Push to git immediately after each crate's tests pass
2. Never remove existing code — only extend
3. Single source of truth — one type, one crate
4. `Reducer<E>` is the only state mutator
5. No `SystemTime::now()` in business logic
6. UI reads projections only — never the ledger
7. `cargo clippy --workspace -- -D warnings` must pass before any push
