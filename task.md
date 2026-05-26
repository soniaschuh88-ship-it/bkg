# BKG — task.md · Full DELPHOS Specification

> **System ethic**: *Single source of truth. One module, one location.*
> **Vision**: BKG is not a task app. It is a deterministic OS for causal agent orchestration
> with replayable reality simulation.

---

## Ontology

```
Event Ledger  →  Causa Reconstruction  →  Katoptron Projection  →  UI
```

The UI is only the visible shadow of event causality.
Nothing exists outside the event ledger. No UI state. No hidden mutations.

---

## Current Workspace State

**24 crates, 130+ Rust files, 6 git commits** — foundation complete.

All 9 BKG invariants are enforced by the existing crates:

| # | Invariant | Enforced by |
|---|---|---|
| 1 | No mutation without event | `bkg-event` — append-only ledger |
| 2 | No direct realm writes | `bkg-kernel` — RealmRouter |
| 3 | No state outside ledger | `bkg-swd` + `bkg-capsule` |
| 4 | Replay must be identical | `bkg-replay` + BLAKE3 hash chain |
| 5 | One file = one responsibility | workspace convention |
| 6 | Drift detection | `bkg-verifier` |
| 7 | Realm isolation | `bkg-contracts` — CausalContract only |
| 8 | SWD is the audit protocol | `bkg-swd` |
| 9 | Single source of truth | workspace structure |

---

## Full DELPHOS Layout — Target Architecture

```
bkg/
└── delphos/
    ├── cognition/                        ← System law and causality
    │   ├── core/         bkg-core        DONE  typed IDs, Hash256, BkgError
    │   ├── kernel/       bkg-kernel      DONE  Genesis, RealmRouter, ContractValidator
    │   ├── event/        bkg-event       DONE  Event, EventLedger (mem+file)
    │   ├── contracts/    bkg-contracts   DONE  CausalContract (cross-realm only)
    │   ├── protocol/     bkg-acp         DONE  ACP JSON-RPC 2.0 + AgentBridge
    │   ├── clock/        bkg-clock       NEW   Realm Clock — deterministic ticks
    │   ├── project/      bkg-project     NEW   Project registry + settings
    │   └── workflow/     bkg-workflow    NEW   Plan→Review→Execute gates + verdicts
    │
    └── domains/
        ├── thalassa/                     ← Execution realm
        │   ├── runtime/      bkg-runtime     DONE  AgentRuntime (Telum sandbox)
        │   ├── orchestrator/ bkg-orchestrator DONE  TaskGraph, EventBus, Scheduler
        │   ├── providers/    bkg-providers   DONE  13 LLM providers (pi-free)
        │   ├── agents/       bkg-agents      DONE  7 agents + credentials
        │   ├── session/      bkg-session     DONE  SessionManager + UniversalEvent
        │   ├── exec/         bkg-exec        DONE  bash, file, grep, glob
        │   ├── task/         bkg-task        NEW   Task capsules + lifecycle
        │   ├── mission/      bkg-mission     NEW   Mission→Milestone→Slice→Feature→Task
        │   ├── scheduler/    bkg-scheduler   NEW   Deterministic DAG + priority queue
        │   ├── chat/         bkg-chat        NEW   Chat rooms + mailbox + streaming
        │   ├── github/       bkg-github      NEW   Issue import + PR creation + OAuth
        │   ├── vm/           bkg-vm          NEW   Tool execution sandbox VM
        │   └── providers/abi/ (in bkg-providers) NEW  Model ABI layer
        │
        ├── arche/                        ← Persistence realm
        │   ├── capsule/  bkg-capsule     DONE  Capsule + CapsuleManager
        │   ├── store/    bkg-store       DONE  sled + in-memory
        │   └── mesh/     bkg-mesh        NEW   Multi-node replication + lease mgmt
        │
        ├── styx/                         ← Event provider realm
        │   ├── provider/ bkg-swd         DONE  SwdEngine
        │   ├── tools/    bkg-tools       DONE  ledger_summary
        │   └── lanes/    bkg-lanes       NEW   Realm Bus / IPC fabric
        │
        ├── katoptron/                    ← Observation and projection realm
        │   ├── crypto/   bkg-crypto      DONE  BLAKE3, Ed25519
        │   ├── verifier/ bkg-verifier    DONE  hash-chain + PermissionEnforcer
        │   ├── telemetry/ bkg-telemetry  DONE  model call tracking + quota
        │   ├── approval/ bkg-approval    NEW   Approval gates + audit trail
        │   ├── secrets/  bkg-secrets     NEW   AES-256-GCM secrets + policies
        │   ├── eval/     bkg-eval        NEW   Scorecards + evidence + batches
        │   ├── plugins/  bkg-plugins     NEW   Plugin discovery + manifest loader
        │   ├── ecs/      bkg-ecs         NEW   Entity-Component-System (world)
        │   ├── compiler/ bkg-compiler    NEW   Katoptron UI compiler pipeline
        │   └── physics/  bkg-physics     NEW   DAG physics engine (mass, tension)
        │
        ├── anamnesis/                    ← Policy and memory realm
        │   ├── policy/   bkg-policy      DONE  PolicyEngine + built-in policies
        │   ├── memory/   bkg-memory      DONE  MemoryGraph (dreams, insights)
        │   └── operator/ bkg-operator    NEW   Operator consciousness layer
        │
        ├── mnemos/                       ← Replay and reconstruction realm
        │   ├── memory/   bkg-memory      DONE  causal graph + decay
        │   └── replay/   bkg-replay      DONE  ReplayEngine, divergence detection
        │
        └── speculum/                     ← Audit and verification realm
            ├── capabilities/ bkg-capabilities NEW  Realm permissions + capability tokens
            ├── diff/         bkg-diff        NEW   Reality diff engine
            └── recovery/     bkg-recovery    NEW   Crash reconstruction + rollback

    └── reflection/
        ├── ui/
        │   └── atlantean/  bkg-atlantean  DONE+EXT  cyberpunk/Atlantis dashboard
        └── inspector/      bkg-inspector  DONE  realm name registry

    └── threshold/
        └── cli/            bkg-cli        DONE+EXT  all commands

    └── calibration/
        └── testing/        bkg-testing    DONE  shared test fixtures
```

---

## Crates — Complete Table (42 total after implementation)

### DONE (24 crates)

| Crate | Description |
|---|---|
| `bkg-core` | typed IDs, Hash256, BkgError, ExecutionSeed |
| `bkg-crypto` | BLAKE3, Ed25519, key derivation |
| `bkg-event` | Event, hash-chained EventLedger, LaneEvent |
| `bkg-contracts` | CausalContract — cross-realm only |
| `bkg-kernel` | Genesis lock, RealmRouter, ContractValidator |
| `bkg-swd` | SwdEngine — audit protocol lifecycle |
| `bkg-capsule` | Capsule + CapsuleManager — versioned containers |
| `bkg-store` | InMemoryStore + SledStore |
| `bkg-memory` | MemoryGraph — impact × recurrence × depth |
| `bkg-replay` | ReplayEngine, DivergenceDetector |
| `bkg-verifier` | hash-chain + PermissionEnforcer |
| `bkg-policy` | PolicyEngine + built-in policies |
| `bkg-runtime` | AgentRuntime (Telum sandbox) |
| `bkg-orchestrator` | TaskGraph (DAG), EventBus, Scheduler |
| `bkg-exec` | bash, file, grep, glob tools |
| `bkg-tools` | ledger_summary, dump_realm |
| `bkg-inspector` | realm name registry |
| `bkg-providers` | 13 LLM providers (pi-free port) |
| `bkg-telemetry` | model call tracking + quota monitor |
| `bkg-agents` | 7 agents + credentials + status (sandbox-agent) |
| `bkg-session` | SessionManager + UniversalEvent (sandbox-agent) |
| `bkg-acp` | ACP JSON-RPC 2.0 + InferenceProxy (sandbox-agent) |
| `bkg-atlantean` | cyberpunk/Atlantis web dashboard |
| `bkg-cli` | `bkg` binary — all commands |

### NEW — Phase 1 (Core Infrastructure)

| Crate | Location | Purpose |
|---|---|---|
| `bkg-clock` | `cognition/clock` | **Realm Clock**: deterministic ticks, vector clocks, causal ordering, replay-safe timestamps. No `SystemTime::now()`. |
| `bkg-project` | `cognition/project` | Project registry (`~/.bkg/bkg-central.db`), settings, isolation, path mapping |
| `bkg-workflow` | `cognition/workflow` | Plan→Review→Execute→Review cycle, quality gates, verdicts (APPROVE/REVISE/RETHINK/UNAVAILABLE), wave execution |
| `bkg-task` | `domains/thalassa/task` | Task capsules (`.bkg/tasks/{id}/`), lifecycle, kanban, DAG dependencies, stuck detection |
| `bkg-mission` | `domains/thalassa/mission` | Mission→Milestone→Slice→Feature→Task, autopilot, fix-budget retries |
| `bkg-scheduler` | `domains/thalassa/scheduler` | Deterministic DAG scheduler, priority queue, overlap gating, dependency unblock fanout |
| `bkg-lanes` | `domains/styx/lanes` | Realm Bus / IPC fabric: priority lanes, replayable packets, backpressure, signatures |

### NEW — Phase 2 (Security & Features)

| Crate | Location | Purpose |
|---|---|---|
| `bkg-secrets` | `domains/katoptron/secrets` | AES-256-GCM secrets, project/global scopes, per-secret policies, OS keychain |
| `bkg-approval` | `domains/katoptron/approval` | Approval gates, immutable audit trail, deduplication, action classification |
| `bkg-eval` | `domains/katoptron/eval` | Task scorecards, evidence collection, scheduled batches, follow-ups |
| `bkg-capabilities` | `domains/speculum/capabilities` | Realm-scoped permissions, temporary grants, signed execution scopes |
| `bkg-plugins` | `domains/katoptron/plugins` | Plugin discovery, YAML manifest, UI slots, prompt contributions |
| `bkg-chat` | `domains/thalassa/chat` | Chat rooms, multi-agent messaging, mailbox, streaming SSE, attachments |
| `bkg-github` | `domains/thalassa/github` | Issue import, PR creation, OAuth, branch pushes, status badges |

### NEW — Phase 3 (Advanced Systems)

| Crate | Location | Purpose |
|---|---|---|
| `bkg-mesh` | `domains/arche/mesh` | Multi-node replication, mDNS discovery, lease management, write queue + replay |
| `bkg-vm` | `domains/thalassa/vm` | Tool execution sandbox VM, syscall layer, filesystem virtualization, snapshot rollback |
| `bkg-ecs` | `domains/katoptron/ecs` | Entity-Component-System, sparse archetypes, deterministic updates, replay-safe world |
| `bkg-physics` | `domains/katoptron/physics` | DAG physics engine: node mass, edge tension, entropy propagation, structural collapse |
| `bkg-compiler` | `domains/katoptron/compiler` | Katoptron UI compiler: AST → Geometry → Render graph → Bytecode |
| `bkg-operator` | `domains/anamnesis/operator` | Operator consciousness: intent tracking, contextual weighting, adaptive orchestration |
| `bkg-diff` | `domains/speculum/diff` | Reality diff: state/graph/capsule/entropy/timeline diffs, causal mutation traces |
| `bkg-recovery` | `domains/speculum/recovery` | Crash reconstruction, partial replay repair, corrupted capsule healing |

---

## Phase 1: Core Infrastructure Detail

### bkg-clock — Realm Clock
```
clock.rs         — RealmClock struct, tick() → SequencedInstant
tick.rs          — Tick: lamport_ts + wall_time + realm_id
timeline.rs      — Timeline: ordered sequence of ticks
epoch.rs         — Epoch: genesis tick + current tick
causal_time.rs   — CausalTime: vector clock per realm
```

No `SystemTime::now()` anywhere. Deterministic by construction.
A `SequencedInstant` is: `(realm_id, lamport_counter, wall_nanos_for_display_only)`.
Replay safety: two events with equal lamport counters in the same realm = determinism failure.

### bkg-task — Task Capsules
```
capsule/
  mod.rs         — TaskCapsule: .bkg/tasks/{id}/ filesystem layout
  ledger.rs      — per-task event ledger (extends bkg-event)
  state.rs       — TaskState reconstructed from events
  snapshot.rs    — Snapshot: frozen task state for archival
lifecycle/
  mod.rs         — TaskLifecycle: planning→todo→in-progress→review→done→archived
  status.rs      — TaskStatus enum + valid transitions
  tags.rs        — labels, priorities, dependencies
dag/
  mod.rs         — DependencyGraph: DAG with cycle detection
  topological.rs — topological sort for execution ordering
task.rs          — Task struct: id, title, status, deps, capsule_path, prompt_md
```

**Task Capsule filesystem layout** (mirrors `.fusion/tasks/{id}/`):
```
.bkg/tasks/TASK-{id}/
  state.json          ← current reconstructed state (cached)
  ledger/             ← append-only event files (extends bkg-event)
  diffs/              ← git diffs from each execution step
  memory/             ← agent memory for this task
  snapshots/          ← frozen capsule snapshots
  prompt.md           ← AI-generated execution plan (PROMPT.md equivalent)
  logs/               ← execution logs per step
```

### bkg-workflow — Plan→Review→Execute Gates
```
gate.rs         — WorkflowGate: phase + verdict + retry config
verdict.rs      — Verdict: APPROVE | REVISE | RETHINK | UNAVAILABLE
phase.rs        — WorkflowPhase: Plan | PlanReview | Execute | ExecuteReview
wave.rs         — WaveExecution: parallel steps within a task
step.rs         — WorkflowStep: template with configurable phases
engine.rs       — WorkflowEngine: orchestrates gate transitions
```

**Reviewer fallback**: UNAVAILABLE → retry with fallback model + stricter instructions.
**Wave execution**: parallel step sessions within a task (independent file scopes).
**Pre/post-merge gates**: pre = blocks merge, post = informational only.

### bkg-mission — Mission Hierarchy
```
mission.rs      — Mission: id + title + description + status
milestone.rs    — Milestone: ordered set of slices
slice.rs        — Slice: parallel feature set with shared scope
feature.rs      — Feature: acceptance criteria + task_id + retry budget
autopilot.rs    — MissionAutopilot: auto-activate next slice on completion
contracts.rs    — MissionContract: assertions + success criteria
```

### bkg-scheduler — Deterministic DAG Scheduler
```
scheduler.rs    — TaskScheduler: priority queue + slot semaphore
dag.rs          — DAG: topological sort + cycle detection
priority.rs     — Priority: Urgent > High > Normal > Low + FIFO tie-break
overlap.rs      — OverlapGate: prevent tasks with shared file scope from running concurrently
lease.rs        — TaskLease: distributed lease with epoch fencing
blocker.rs      — BlockerTracker: sticky blockers + unblock fanout
```

---

## Phase 2: Security & Features Detail

### bkg-secrets — Encrypted Secrets Store
```
store.rs        — SecretsStore: AES-256-GCM, per-row nonce, SQLite backend
secret.rs       — Secret: name + value + scope + policy + provenance
scope.rs        — SecretScope: Project(id) | Global
policy.rs       — AccessPolicy: Auto | Prompt | Deny
keychain.rs     — MasterKeyProvider: OS keychain + ~/.bkg/master.key fallback
export.rs       — SecretExport: materialize secrets as env vars for worktree
```

### bkg-approval — Approval Gates + Audit
```
gate.rs         — ApprovalGate: pending → approved/denied → completed
audit.rs        — ApprovalAudit: append-only audit trail (bkg-event based)
dedup.rs        — ApprovalDedup: dedupe by action context key
request.rs      — ApprovalRequest: kind + description + context + risk
policy.rs       — ActionPolicy: allow | block | require-approval
notification.rs — pause task with reason="awaiting-approval"
```

### bkg-eval — Task Evaluations
```
scorecard.rs    — Scorecard: categories + weights + bands
evidence.rs     — EvalEvidence: signals + AI scoring + suggestions
batch.rs        — EvalBatch: scheduled evaluation on completed-task windows
persistence.rs  — EvalResult: durable with task snapshots
followup.rs     — FollowUp: normalized suggestions with suppression/dedup
```

### bkg-chat — Chat Rooms + Mailbox
```
room.rs         — ChatRoom: id + members + responder config
message.rs      — ChatMessage: sender + content + attachments + metadata
mailbox.rs      — Mailbox: per-user/agent inbox for notifications
streaming.rs    — SSE streaming for real-time chat delivery
attachment.rs   — FileAttachment: metadata + upload path
mention.rs      — Mention routing: direct responders + ambient responders
```

---

## Phase 3: Advanced Systems Detail

### bkg-mesh — Multi-Node Replication
```
node.rs         — MeshNode: id + address + capabilities + health
discovery.rs    — NodeDiscovery: mDNS + central registry
lease.rs        — MeshLease: epoch fencing + abandoned-lease recovery
sync.rs         — StateSync: replicate task/mission/agent state
write_queue.rs  — WriteQueue: retryable writes queued for replay on peer recovery
snapshot.rs     — MeshSnapshot: checkpoint topology for recovery
```

### bkg-vm — Tool Execution Sandbox VM
```
vm.rs           — SandboxVm: deterministic execution environment
syscalls.rs     — SyscallLayer: virtualized, replay-safe I/O
mounts.rs       — VfsMount: scoped filesystem access
limits.rs       — ResourceLimits: memory + CPU + time caps
snapshot.rs     — VmSnapshot: rollback point before tool execution
process.rs      — VmProcess: deterministic child process management
```

### bkg-ecs — Entity Component System
```
entity.rs       — Entity: u64 ID, generation counter
component.rs    — Component trait + storage
archetype.rs    — Archetype: sparse-set column storage
world.rs        — World: entity registry + component stores + systems
query.rs        — Query: filtered entity iteration
system.rs       — System trait: deterministic update, replay-safe
```

### bkg-physics — DAG Physics Engine
```
mass.rs         — NodeMass: derived from task complexity + dependency count
tension.rs      — EdgeTension: derived from dependency criticality
entropy.rs      — EntropyState: system disorder measurement
simulation.rs   — PhysicsSimulation: n-body layout, deterministic
forces.rs       — Force: gravity, spring, damping
stability.rs    — StabilityThreshold: collapse detection + alert
```

### bkg-compiler — Katoptron UI Compiler Pipeline
```
ast.rs          — UiAst: abstract UI tree from realm state
compiler.rs     — UiCompiler: EventLedger → UiAst → Bytecode
bytecode.rs     — UiBytecode: portable render instructions
frame.rs        — UiFrame: rendered output for one tick
geometry.rs     — GeometryGraph: layout from physics simulation
render_graph.rs — RenderGraph: dependency-aware render pass ordering
```

Pipeline:
```
Event Ledger → Realm State → UI AST → Geometry (Physics) → Render Graph → Bytecode → Frame
```

### bkg-lanes — Realm Bus / IPC Fabric
```
realm_bus.rs    — RealmBus: deterministic inter-realm transport
lane.rs         — Lane: priority-classified message channel
packet.rs       — BusPacket: signed + sequenced + replayable
router.rs       — LaneRouter: routes by (source_realm, target_realm, priority)
qos.rs          — QosPolicy: latency guarantees per lane class
backpressure.rs — BackpressureController: congestion handling
```

---

## Atlantean Dashboard — Extended UI Pages

All existing pages remain. New pages:

| Page | Route | Description |
|---|---|---|
| **Kanban** | `/kanban` | Drag-drop columns: planning/todo/in-progress/in-review/done |
| **Task Detail** | `/tasks/:id` | PROMPT.md viewer, logs, diffs, workflow steps, capsule events |
| **Missions** | `/missions` | Mission hierarchy browser with autopilot controls |
| **Scheduler** | `/scheduler` | DAG visualization with physics layout |
| **Secrets** | `/secrets` | Encrypted secrets management |
| **Approvals** | `/approvals` | Pending approval queue with audit trail |
| **Chat** | `/chat` | Chat rooms + mailbox |
| **Evals** | `/evals` | Scorecard results + evidence viewer |
| **Mesh** | `/mesh` | Node health + replication status |
| **Plugins** | `/plugins` | Plugin discovery + install |
| **Reality Diff** | `/diff` | Timeline diff + divergence visualization |
| **Physics** | `/physics` | Live DAG physics simulation |

---

## CLI Commands — Full Fusion + BKG

```bash
# Task
bkg task create --title "..." --desc "..."
bkg task plan <id>              # AI planning agent → PROMPT.md
bkg task show <id>
bkg task logs <id> [--follow]
bkg task steer <id> "..."       # guide mid-execution
bkg task archive <id>

# Mission
bkg mission create --title "..."
bkg mission show <id>
bkg mission activate-slice <id> <slice>

# Project
bkg project add <path>
bkg project list
bkg project default <id>

# Workflow
bkg workflow approve <task-id>
bkg workflow revise <task-id> "feedback"
bkg workflow rethink <task-id>

# Secrets
bkg secret set <name> <value> [--project <id>]
bkg secret get <name>
bkg secret list
bkg secret delete <name>

# Agents
bkg agent list
bkg agent spawn --name X --mode bkg_supervised
bkg agent show <id>

# Providers (pi-free)
bkg providers list
bkg providers models <id>
bkg providers toggle <id>
bkg providers refresh <id|all>

# Sessions (sandbox-agent)
bkg session list
bkg session create --agent claude --mode bkg_supervised
bkg session send <id> "..."
bkg session inspect <id>

# Chat
bkg chat                         # start interactive LLM chat
bkg chat --prompt "..."          # non-interactive

# Settings
bkg settings set <key> <value>
bkg settings export

# Dashboard
bkg dashboard                    # start atlantean server
bkg serve                        # headless daemon mode

# Evals
bkg eval run <task-id>
bkg eval list

# Mesh
bkg mesh status
bkg mesh nodes

# Plugins
bkg plugins list
bkg plugins install <package>
```

---

## Fusion Features → DELPHOS Mapping

### Task Lifecycle (9 features)
| Feature | Crate | Status |
|---|---|---|
| Task Creation | `bkg-task` | TODO |
| Task Planning (AI → PROMPT.md) | `bkg-task` + `bkg-workflow` | TODO |
| Task Status Tracking | `bkg-task` | TODO |
| Task Columns (Kanban) | `bkg-atlantean` | TODO |
| Task Dependencies (DAG) | `bkg-task` + `bkg-scheduler` | TODO |
| Task Archiving | `bkg-task` + `bkg-store` | TODO |
| Task Refinement | `bkg-task` + `bkg-workflow` | TODO |
| Task Comments | `bkg-chat` | TODO |
| Task Search | `bkg-task` + `bkg-store` | TODO |

### Workflow Execution (12 features)
| Feature | Crate | Status |
|---|---|---|
| AI Planning Agent | `bkg-workflow` + `bkg-acp` | TODO |
| Step-by-Step Execution | `bkg-workflow` | TODO |
| Parallel Wave Execution | `bkg-workflow` + `bkg-scheduler` | TODO |
| Pre/Post-Merge Gates | `bkg-workflow` | TODO |
| Workflow Step Templates | `bkg-workflow` | TODO |
| Workflow Step Phases | `bkg-workflow` | TODO |
| Review Verdicts | `bkg-workflow` | TODO |
| Reviewer Fallback | `bkg-workflow` + `bkg-providers` | TODO |
| Merge Strategy | `bkg-workflow` + `bkg-github` | TODO |
| Merge Conflict Arbitration | `bkg-workflow` + `bkg-github` | TODO |

### Git & Worktree (7 features)
| Feature | Crate | Status |
|---|---|---|
| Isolated Worktrees | `bkg-task` (capsule) | TODO |
| Zero Conflicts | `bkg-scheduler` (overlap gate) | TODO |
| Branch Naming | `bkg-task` | TODO |
| Commit Attribution | `bkg-task` + `bkg-github` | TODO |
| Worktree Metadata | `bkg-task` | TODO |
| Squash Merge | `bkg-workflow` + `bkg-github` | TODO |

### Agent Management (10 features)
| Feature | Crate | Status |
|---|---|---|
| Agent Creation | `bkg-agents` | DONE |
| Agent Presets | `bkg-agents` | DONE |
| Agent Heartbeat | `bkg-agents` + `bkg-orchestrator` | PARTIAL |
| Agent Reflection | `bkg-memory` | PARTIAL |
| Agent Permissions | `bkg-verifier` + `bkg-capabilities` | PARTIAL |
| Agent Ratings | `bkg-eval` | TODO |
| Spawn Agents | `bkg-agents` + `bkg-runtime` | DONE |
| Agent Companies | `bkg-agents` | TODO |
| Agent Mailbox | `bkg-chat` | TODO |
| Agent Approval Gates | `bkg-approval` | TODO |

### AI Models (12 features)
| Feature | Crate | Status |
|---|---|---|
| Dual-Scope Model Hierarchy | `bkg-providers` + `bkg-project` | PARTIAL |
| Executor/Planning/Validator lanes | `bkg-workflow` + `bkg-acp` | TODO |
| Per-Task Overrides | `bkg-task` | TODO |
| All provider support | `bkg-providers` | DONE |
| OAuth Authentication | `bkg-providers` + `bkg-atlantean` | PARTIAL |
| Custom Providers | `bkg-providers` | TODO |
| Fallback Models | `bkg-workflow` | TODO |
| Model Rate Limiting | `bkg-providers` + `bkg-telemetry` | PARTIAL |

### Mission & Planning (10 features)
| Feature | Crate | Status |
|---|---|---|
| Mission Hierarchy | `bkg-mission` | TODO |
| Mission Creation | `bkg-mission` | TODO |
| Milestone Planning | `bkg-mission` | TODO |
| Feature Slices | `bkg-mission` | TODO |
| Feature-to-Task Handoff | `bkg-mission` + `bkg-task` | TODO |
| Mission Autopilot | `bkg-mission` | TODO |
| Mission Validation | `bkg-mission` + `bkg-workflow` | TODO |
| Fix-Feature Retries | `bkg-mission` | TODO |
| Mission Contracts | `bkg-mission` | TODO |

### Multi-Node Mesh (10 features)
| Feature | Crate | Status |
|---|---|---|
| Node Discovery | `bkg-mesh` | TODO |
| Node Health Monitoring | `bkg-mesh` | TODO |
| Task Mesh Sync | `bkg-mesh` | TODO |
| Mission Sync | `bkg-mesh` | TODO |
| Mesh Lease Management | `bkg-mesh` | TODO |
| Distributed Task IDs | `bkg-mesh` + `bkg-task` | TODO |
| Write Queue + Replay | `bkg-mesh` | TODO |

### (All 258+ features covered — see DELPHOS locations above)

---

## New Core Systems (15 systems from vision)

| # | System | Crate | Priority |
|---|---|---|---|
| 1 | Realm Clock | `bkg-clock` | **CRITICAL** |
| 2 | Realm Bus / IPC Fabric | `bkg-lanes` | HIGH |
| 3 | DAG Physics Engine | `bkg-physics` | HIGH |
| 4 | Katoptron UI Compiler | `bkg-compiler` | HIGH |
| 5 | Entity Component System | `bkg-ecs` | HIGH |
| 6 | Realm Capabilities | `bkg-capabilities` | HIGH |
| 7 | Model ABI Layer | in `bkg-providers/abi` | HIGH |
| 8 | Tool Execution VM | `bkg-vm` | HIGH |
| 9 | Genesis Snapshot | in `bkg-kernel` (extend) | MEDIUM |
| 10 | Reality Diff Engine | `bkg-diff` | MEDIUM |
| 11 | Atlantean UI Panels | extend `bkg-atlantean` | HIGH |
| 12 | Operator Consciousness | `bkg-operator` | MEDIUM |
| 13 | Realm Recovery | `bkg-recovery` | MEDIUM |
| 14 | Telemetry Physics | extend `bkg-telemetry` | MEDIUM |
| 15 | Terminal Render Backends | `bkg-compiler/backends` | MEDIUM |

---

## Implementation Order

### Batch 1 — Clock + Foundation (push immediately after each)
1. `bkg-clock` — Realm Clock with vector clocks
2. `bkg-project` — Project registry + settings
3. `bkg-workflow` — Plan→Review→Execute + verdicts
4. `bkg-task` — Task capsules + lifecycle + DAG

### Batch 2 — Scheduling + Mission (push immediately after each)
5. `bkg-scheduler` — Deterministic DAG scheduler
6. `bkg-mission` — Mission hierarchy + autopilot
7. `bkg-lanes` — Realm Bus IPC fabric

### Batch 3 — Security + Features (push immediately after each)
8. `bkg-secrets` — AES-256-GCM secrets store
9. `bkg-approval` — Approval gates + audit trail
10. `bkg-capabilities` — Realm permission tokens
11. `bkg-eval` — Task scorecards
12. `bkg-chat` — Chat rooms + mailbox
13. `bkg-github` — GitHub integration

### Batch 4 — Advanced Systems (push immediately after each)
14. `bkg-mesh` — Multi-node replication
15. `bkg-ecs` — Entity Component System
16. `bkg-physics` — DAG physics
17. `bkg-compiler` — Katoptron UI compiler
18. `bkg-vm` — Tool execution sandbox VM
19. `bkg-diff` — Reality diff engine
20. `bkg-recovery` — Crash reconstruction
21. `bkg-operator` — Operator consciousness

### Batch 5 — UI + CLI (push immediately after each)
22. Atlantean: Kanban + Task detail + Mission view + Chat + Diff
23. CLI: all Fusion commands (`bkg task`, `bkg mission`, etc.)
24. Terminal render backends (ratatui + headless)

---

## Deterministic OS Invariants

BKG implements these in Rust at compile time where possible:

1. **No mutation without event** → `bkg-event` emit-or-panic design
2. **No direct realm writes** → `bkg-lanes` RealmBus enforces routing
3. **No state outside ledger** → `bkg-task` capsules reconstruct from events
4. **Replay must be identical** → `bkg-clock` vector clocks + BLAKE3 chain
5. **Drift = failure** → `bkg-verifier` + `bkg-diff` detect all deviations
6. **VM isolation** → `bkg-vm` seals tool execution
7. **Causal ordering** → `bkg-clock` CausalTime per realm
8. **One file = one responsibility** → enforced by workspace layout
9. **Single source of truth** → no type defined in more than one crate

---

*BKG v0.1.0 — DELPHOS architecture*
*Target: 42 crates · 258+ Fusion features · 15 new core systems*
*Single source of truth. One module, one location.*
