# BKG — FEATURES.md
## Complete Feature Inventory by System

> **Single source of truth. One module, one location.**
> Status: ✅ DONE · 🔨 IN PROGRESS · 📋 PLANNED

---

## 1. Core Foundation (cognition/)

### bkg-core ✅
- Typed IDs: `TaskId`, `AgentId`, `RealmId`, `SessionId`, `MeshNodeId`
- `Hash256` — BLAKE3 digest wrapper
- `ExecutionSeed` — deterministic seed derivation
- `BkgError` + `BkgResult` — unified error type
- `Capability` enum — `RuntimeExecute`, `CapsuleWrite`, `LedgerRead`, etc.
- `RealmId` registry

### bkg-kernel ✅
- `Genesis` — lock + verify + locked_hash
- `RealmRouter` — contract-based cross-realm routing
- `CausalContractValidator` — validates before forwarding
- **📋 Kernel Arbitrator** — prevents concurrent causality corruption, invalid realm transitions, replay paradoxes

### bkg-event ✅
- `Event` — id, realm_id, event_type, payload, hash, prev_hash
- `EventLedger` trait — `emit()`, `len()`, `iter_from()`
- `InMemoryLedger` — fast in-process ledger
- `FileLedger` — append-only JSONL file ledger
- **📋 `DomainEvent<T>`** — typed generic events for compile-time replay validation

### bkg-contracts ✅
- `CausalContract` — the only legal cross-realm message
- `ContractPayload` — typed payload wrapper

### bkg-protocol (bkg-acp) ✅
- JSON-RPC 2.0: `RpcRequest`, `RpcResponse`, `RpcError`
- 24 ACP methods: `session/*`, `agent/*`, `process/*`, `file/*`, `_bkg/*`
- `AgentBridge` — native agent stdout → `UniversalEventData` translation
- `InferenceProxy` — provider fallback chain routing

---

## 2. New Critical Architecture (Batch 0) 📋

### bkg-state (cognition/state) 📋 CRITICAL
- `RealmStateMachine` — single canonical state reducer
- `apply(state, event) -> Result<RealmState>` — ONLY allowed state mutator
- `Projection` — event → read model (UI reads projections ONLY)
- `RealmState` — full reconstructed realm state
- `StateTransition` — valid/invalid transition classification
- `StateMutation` — typed mutation record
- `StateSnapshot` — frozen realm state for archival
- `Reconciliation` — repair partial writes / replay mismatches / corrupted capsules

### bkg-abi (cognition/abi) 📋 CRITICAL
- Universal Realm ABI — standardizes ALL inter-system communication
- Event ABI — typed event serialization contract
- Packet ABI — IPC packet format (used by bkg-lanes)
- Capsule ABI — capsule serialization contract
- Projection ABI — read-model wire format
- UI Bytecode ABI — render instruction format
- Mesh Sync ABI — cross-node replication format
- Plugin ABI — plugin contribution format
- Provider ABI — LLM request/response normalization

### bkg-clock (cognition/clock) 📋 CRITICAL
- `RealmClock` — deterministic tick source (NO `SystemTime::now()`)
- `SequencedInstant` — `(realm_id, lamport_counter, wall_nanos_display_only)`
- `VectorClock` — per-realm causality tracking
- `CausalTime` — total causal ordering across realms
- `Timeline` — ordered sequence of ticks
- `Epoch` — genesis tick + current tick
- Divergence detection: equal lamport in same realm = determinism failure

### bkg-state-machine (capsule lifecycle) 📋
- Capsule states: `Created → Mounted → Active → Frozen → Forked → Archived → Corrupted → Recovered`
- Lifecycle transitions with invariant checks
- Integration with bkg-vm for execution sealing

---

## 3. Persistence (arche/)

### bkg-capsule ✅
- `Capsule` — immutable versioned state container
- `CapsuleManager` — create, open, snapshot, verify
- `CapsuleVersion` — version hash + parent hash
- **📋 Lifecycle State Machine** — Created/Mounted/Active/Frozen/Forked/Archived/Corrupted/Recovered

### bkg-store ✅
- `InMemoryStore` — in-process key-value store
- `SledStore` — persistent sled-backed store
- Generic `StorageBackend` trait

### bkg-snapshot (speculum/snapshot) 📋
- `GenesisSnapshot` — frozen initial world state
- `RealitySnapshot` — full deterministic world snapshot
- `TimelineSnapshot` — per-timeline frozen state
- Compression + export + restore
- CLI: `bkg snapshot create/fork/export/diff/restore`

### bkg-mesh (arche/mesh) 📋
- `MeshNode` — id + address + capabilities + health
- `NodeDiscovery` — mDNS + central registry
- `MeshLease` — epoch fencing + abandoned-lease recovery
- `StateSync` — replicate task/mission/agent state across nodes
- `WriteQueue` — retryable writes queued for replay when peer recovers
- `MeshSnapshot` — checkpoint topology for recovery
- Degraded reads — fallback to last-known stale data

---

## 4. Execution Realm (thalassa/)

### bkg-runtime ✅
- `AgentRuntime` — Telum sandbox with SWD recording
- `Capability`-based permission enforcement
- Agent spawn + task execute + SWD commit

### bkg-orchestrator ✅
- `TaskGraph` — DAG with cycle detection
- `EventBus` — async pub/sub for cross-component events
- `Scheduler` — dependency-aware task execution

### bkg-exec ✅
- `BashTool` — execute shell commands + capture stdout/stderr
- `ReadFileTool`, `WriteFileTool`, `EditFileTool` — path-scoped file operations
- `GrepTool`, `GlobTool` — file search
- `ToolRegistry` — permission-gated dispatch

### bkg-agents ✅ (sandbox-agent port)
- 7 agent IDs: `Claude`, `Codex`, `Opencode`, `Amp`, `Pi`, `Cursor`, `Mock`
- `AgentMode`: Default | Bypass | PlanMode | **BkgSupervised** (new — enforces workflow gates)
- `AgentInfo` — capabilities per agent (streaming, permissions, file_ops, images)
- `AgentCredentials` — fallback chain (user → admin → env → agent config)
- `AgentStatus` — live binary probe + credential check
- `InstallOptions` / `InstallResult` — npm/url/system install recipes

### bkg-session ✅ (sandbox-agent port)
- `BkgSession` — conversation with broadcast streaming + offset replay
- `SessionState` — Pending/Running/AwaitingPermission/Paused/Finished/Error
- `UniversalEvent` — 10 event types, agent-agnostic
- `UniversalMessage` — Parsed/Unparsed + 8 part types (text, tool_call, thinking, file, image...)
- `PermissionRequest/Response` — human-in-the-loop flows
- `PermissionStrategy` — AlwaysPrompt / AutoApproveSafe / AutoApproveAll / AutoDenyAll
- `SessionManager` — in-memory registry
- `SseEvent` — SSE wire format with offset-based replay

### bkg-task (thalassa/task) 📋
- **Task Capsule** filesystem layout: `.bkg/tasks/{id}/ledger/ diffs/ memory/ snapshots/ prompt.md logs/`
- `TaskLifecycle` — planning → todo → in-progress → in-review → done → archived
- `TaskStatus` valid transitions + guards
- `DependencyGraph` — DAG with cycle detection + topological sort
- Task search (full-text across tasks + PROMPT.md)
- Stuck task detection + recovery (loop detection, max retry budget)
- Ghost review recovery (idle in-review tasks)
- Overlap detection (shared file scope across parallel tasks)

### bkg-mission (thalassa/mission) 📋
- `Mission → Milestone → Slice → Feature → Task` hierarchy
- `MissionAutopilot` — auto-activate next slice on feature completion
- `MissionContract` — assertions + success criteria
- `FixBudget` — max retries for failed features
- Roadmap → Mission handoff
- Fix-feature retries within budget

### bkg-scheduler (thalassa/scheduler) 📋
- Deterministic DAG scheduler (topological sort + cycle check)
- Priority queue: Urgent > High > Normal > Low, then FIFO
- `OverlapGate` — blocks tasks with shared file scope from running concurrently
- `TaskLease` — distributed lease with epoch fencing
- `DependencyUnblockFanout` — dispatch unblocked dependents within priority class
- Sticky blockers + manual unblock
- `AgentSemaphore` — concurrent agent slot limiting

### bkg-providers ✅ (pi-free port)
- **13 providers**: Ollama, NVIDIA, OpenRouter, SambaNova, LLM7, Kilo, Cline, ZenMux, CrofAI, Codestral, DeepInfra, Together, Novita
- Tier metadata: free / freemium / paid / private + signup_url
- `FreeModelDetector` — Route A (cost-based) + Route B (name-based)
- `ProviderRegistry` — per-provider toggle (free-only / show-all)
- `ProviderToggleState` — persisted to `~/.bkg/providers-toggle.json`
- CI score enhancer — 17 well-known models
- `fetch_json` + `resolve_key` (env fallback)
- **📋 Model ABI layer** — `providers/abi/`: request/response normalization

### bkg-vm (thalassa/vm) 📋
- `SandboxVm` — deterministic execution environment
- `SyscallLayer` — virtualized, replay-safe I/O
- `VfsMount` — scoped filesystem access
- `ResourceLimits` — memory + CPU + time caps
- `VmSnapshot` — rollback point before tool execution
- `VmProcess` — deterministic child process management

### bkg-chat (thalassa/chat) 📋
- `ChatRoom` — id + members + direct/ambient responder config
- `ChatMessage` — sender + content + attachments + metadata
- `Mailbox` — per-user/agent inbox for system messages + notifications
- SSE streaming for real-time delivery
- `FileAttachment` — metadata + upload path
- Mention routing: direct responders + ambient responders

### bkg-github (thalassa/github) 📋
- Issue import → tasks (with filtering)
- PR creation from tasks
- Real-time PR/issue status badges
- GitHub OAuth
- Branch push + squash/merge/rebase strategy selection

---

## 5. Observation and Projection (katoptron/)

### bkg-crypto ✅
- BLAKE3 — `Hash256`, `hash_bytes()`, `hash_chain()`
- Ed25519 — `SigningKey`, `VerifyingKey`, `sign()`, `verify()`
- Seed derivation — deterministic from `ExecutionSeed`

### bkg-verifier ✅
- `verify_hash_chain()` — full ledger integrity check
- `verify_capsule_chain()` — capsule history check
- `detect_drift()` — expected vs actual hash comparison
- `PermissionEnforcer` — check/check_all with ReadOnly/WorkspaceWrite/DangerFullAccess
- `PermissionRequest` — tool_name + path extraction from JSON input

### bkg-telemetry ✅ (pi-free port)
- `ModelCallRecord` — success/failure constructors with latency + tokens + cost
- `ModelStats` — per-model aggregation (calls, tokens, latency, success_rate)
- `ModelTracker` — in-memory + JSON persistence
- `QuotaMonitor` — per-provider quota tracking + persistence
- **📋 Telemetry Physics** — entropy, pressure, heat, stability, latency as physical properties

### bkg-approval (katoptron/approval) 📋
- `ApprovalGate` — pending → approved/denied → completed
- Immutable audit trail (bkg-event based, append-only)
- `ApprovalDedup` — dedupe by action context key
- `ApprovalRequest` — kind + description + context + risk level
- `ActionPolicy` — allow | block | require-approval
- Task pause notification with `pause_reason="awaiting-approval"`

### bkg-secrets (katoptron/secrets) 📋
- AES-256-GCM encryption, per-row nonce
- `SecretScope` — Project(id) | Global
- `AccessPolicy` — Auto | Prompt | Deny
- OS keychain + `~/.bkg/master.key` fallback
- Secret export as env vars for worktree
- Secret read provenance tracking

### bkg-eval (katoptron/eval) 📋
- `Scorecard` — categories + weights + bands + deterministic scoring
- `EvalEvidence` — signals + AI scoring + follow-up suggestions
- `EvalBatch` — scheduled evaluation on completed-task windows
- `EvalResult` — durable with task snapshots for historical readability
- `FollowUp` — normalized suggestions with suppression/dedup

### bkg-plugins (katoptron/plugins) 📋
- Plugin discovery: `GET /api/plugins/ui-slots`, `GET /api/plugins/dashboard-views`
- YAML-based plugin manifest
- Plugin contributions: UI slots, dashboard views, runtime env vars, prompt contributions
- Plugin loader — dynamic module loading
- Plugin store — global install metadata + per-project runtime state

### bkg-ecs (katoptron/ecs) 📋 CRITICAL (Batch 1.5)
- `Entity` — u64 id + generation counter
- Sparse-set archetype storage
- `World` — entity registry + component stores + system runner
- `Query` — filtered entity iteration
- Deterministic update order (no `HashMap` nondeterminism)
- Replay-safe world state
- **All DELPHOS entities become ECS entities**:
  Task, Mission, Slice, Agent, Session, Approval, Message, WorkflowStep, Provider, MeshNode, Timeline

### bkg-projection (katoptron/projection) 📋
- `ProjectionCache` — materialized read models
- `Materializer` — event → read model conversion
- `CacheIndex` — fast entity lookup
- `Invalidation` — event-driven cache invalidation
- `RealtimeSubscription` — push updates to live UI
- Atlantean, physics, kanban, scheduler all read ONLY from projections

### bkg-physics (katoptron/physics) 📋
- `NodeMass` — derived from task complexity + dependency count
- `EdgeTension` — derived from dependency criticality
- `EntropyState` — system disorder measurement
- `PhysicsSimulation` — deterministic n-body layout
- `Force` — gravity, spring, damping
- `StabilityThreshold` — structural collapse detection

### bkg-compiler (katoptron/compiler) 📋
- `UiAst` — abstract UI tree from realm state
- `UiCompiler` — EventLedger → UiAst → Bytecode
- `UiBytecode` — portable render instructions
- `GeometryGraph` — layout from physics simulation
- `RenderGraph` — dependency-aware render pass ordering
- Backends: ratatui / ANSI / GPU / WebGPU / headless (CI)

### bkg-world (katoptron/world) 📋 ULTIMATE
- `RealityGraph` — the Causal World Model
- Entities + Relations + Causality + Temporal flow + Entropy + Intent + Constraints
- Full integration of: Ledger → Reducer → World Graph → Physics → Compiler → Projection → UI
- `WorldQuery` — BQL (BKG Query Language) execution
- Causal trace — who caused what, when, with which capability

---

## 6. Policy and Memory (anamnesis/)

### bkg-policy ✅
- `PolicyEngine` — evaluate events against rules
- Built-in policies: max_depth, realm_isolation, event_ordering

### bkg-memory ✅
- `MemoryGraph` — weighted causal graph (impact × recurrence × depth)
- `MemoryNode` — concept + weight + timestamp
- `MemoryEdge` — causal link + strength
- Decay + crystallize operations

### bkg-operator (anamnesis/operator) 📋
- `OperatorIntent` — inferred user intent from actions
- `PresenceTracker` — operator focus and attention model
- `InteractionHistory` — causal interaction memory
- Contextual weighting for adaptive orchestration
- Enables: AI adapts its behavior based on what operator has been doing

---

## 7. Replay and Reconstruction (mnemos/)

### bkg-replay ✅
- `ReplayEngine` — reconstruct state from event ledger
- `DivergenceDetector` — expected vs actual hash comparison
- `BranchReport` — deviation analysis
- `ReconstructedState` — cumulative_hash + per_realm_state + event_count

---

## 8. Audit and Verification (speculum/)

### bkg-capabilities (speculum/capabilities) 📋
- `Capability` — realm-scoped permission token
- `CapabilityGrant` — temporary, revocable, signed grant
- `ExecutionScope` — signed scope for sandboxed execution
- `SandboxPolicy` — per-capability sandbox rules
- Prevents agents from becoming "allmächtige Götter"

### bkg-snapshot (speculum/snapshot) 📋
- `GenesisSnapshot` — frozen initial world state
- `RealitySnapshot` — full deterministic world snapshot (all realms)
- `TimelineSnapshot` — per-timeline frozen state
- Fork — create new timeline from snapshot
- CLI: `bkg snapshot create/fork/export/diff/restore`

### bkg-diff (speculum/diff) 📋
- `StateDiff` — before/after realm state comparison
- `GraphDiff` — DAG structural changes
- `CapsuleDiff` — capsule version comparison
- `EntropyDiff` — system disorder delta
- `TimelineDiff` — divergence between replay paths
- Integration with bkg-atlantean: Reality Diff UI page

### bkg-recovery (speculum/recovery) 📋
- Crash recovery — detect + classify failure mode
- Partial replay repair — rebuild from last good checkpoint
- Corrupted capsule healing — reconstruct from events
- Event chain repair — fill gaps in hash chain
- Rollback — restore to last valid snapshot

---

## 9. Realm Bus / IPC Fabric (styx/lanes/)

### bkg-lanes (styx/lanes) 📋
- `RealmBus` — deterministic inter-realm transport
- `Lane` — priority-classified message channel (Critical/High/Normal/Background)
- `BusPacket` — signed + sequenced + replayable
- `LaneRouter` — routes by (source_realm, target_realm, priority)
- `QosPolicy` — latency guarantees per lane class
- `BackpressureController` — congestion handling

---

## 10. Dashboard (reflection/ui/atlantean/)

### bkg-atlantean ✅
**Private/Cloud mode switch:**
- Private: WebLLM (browser WebGPU, CDN-loaded) + Ollama tunnel (`/tunnel/ollama/*`)
- Cloud: 13 free providers through fallback chain

**Current pages:**
- Chat — LLM conversation + /slash commands
- Providers — 13 providers, tier, toggle, signup links
- My Keys — per-user API keys by tier
- Agents — 7 agents, status, credentials, mode badges
- Inspector — Session browser + live SSE event stream
- Dashboard — Stats, telemetry, provider table
- Admin — Global keys, default model, free-only toggle

**Planned pages 📋:**
- Kanban — Drag-drop task columns
- Task Detail — PROMPT.md, logs, diffs, workflow steps, capsule events
- Missions — Mission hierarchy + autopilot controls
- Scheduler — DAG visualization + physics layout
- Secrets — Encrypted secrets management
- Approvals — Pending queue + audit trail
- Chat Rooms — Multi-agent group chat
- Evals — Scorecard results + evidence
- Mesh — Node health + replication status
- Plugins — Discovery + install
- Reality Diff — Timeline diff + divergence visualization
- Physics — Live DAG physics simulation

---

## 11. CLI (threshold/cli/)

### bkg-cli ✅ (partial)
```
Core:      init · run · verify · replay · status · isolate
Chat:      chat [--prompt]
Agents:    agent list/spawn/show
Providers: providers list/models/toggle/refresh/telemetry/quota
Sessions:  (via atlantean REST API)
```

**Planned 📋:**
```
Tasks:     task create/plan/show/logs/steer/archive
Missions:  mission create/show/activate-slice
Projects:  project add/list/default
Workflow:  workflow approve/revise/rethink
Secrets:   secret set/get/list/delete
Evals:     eval run/list
Mesh:      mesh status/nodes
Plugins:   plugins list/install
Snapshot:  snapshot create/fork/export/diff/restore
Settings:  settings set/export
Dashboard: dashboard · serve
```

---

## Feature Count Summary

| System | Done | Planned | Total |
|---|---|---|---|
| Core Foundation | ✅ 4 crates | 📋 3 crates | 7 |
| Execution Realm | ✅ 6 crates | 📋 5 crates | 11 |
| Persistence | ✅ 2 crates | 📋 2 crates | 4 |
| Observation | ✅ 3 crates | 📋 9 crates | 12 |
| Policy/Memory | ✅ 2 crates | 📋 1 crate | 3 |
| Replay | ✅ 2 crates | — | 2 |
| Verification | — | 📋 4 crates | 4 |
| IPC Fabric | — | 📋 1 crate | 1 |
| Dashboard | ✅ 1 (partial) | 📋 extend | 1 |
| CLI | ✅ 1 (partial) | 📋 extend | 1 |
| **TOTAL** | **24 crates** | **+18 crates** | **42** |

---

*BKG v0.1.0 · 42 planned crates · 258+ Fusion features · 15 new core systems*
*Single source of truth. One module, one location.*
