# BKG — FEATURES.md v3
## Complete Feature Inventory

> **Single source of truth. One module, one location.**
> Status: ✅ DONE · 🔨 PARTIAL · 📋 PLANNED · 🔬 RESEARCH

---

## Target Crate Count: 52

### DONE (24)
`bkg-core` `bkg-crypto` `bkg-event` `bkg-contracts` `bkg-kernel`
`bkg-swd` `bkg-capsule` `bkg-store` `bkg-memory` `bkg-replay`
`bkg-verifier` `bkg-policy` `bkg-runtime` `bkg-orchestrator` `bkg-exec`
`bkg-tools` `bkg-inspector` `bkg-providers` `bkg-telemetry`
`bkg-agents` `bkg-session` `bkg-acp` `bkg-atlantean` `bkg-cli`

### PLANNED (+28)

| Batch | Crate | Location | Purpose |
|---|---|---|---|
| **0** | `bkg-state` | `cognition/state` | RealmStateMachine — single canonical reducer |
| **0** | `bkg-abi` | `cognition/abi` | Universal Realm ABI + version negotiation |
| **0** | `bkg-clock` | `cognition/clock` | Vector clocks, causal ordering |
| **0** | `bkg-schema` | `cognition/schema` | EventSchemaRegistry + migration strategies |
| **1** | `bkg-project` | `cognition/project` | Project registry + settings |
| **1** | `bkg-workflow` | `cognition/workflow` | Plan→Review→Execute + ExecutionGraph |
| **1** | `bkg-query` | `cognition/query` | BQL — BKG Query Language engine |
| **1** | `bkg-task` | `domains/thalassa/task` | Task capsules + lifecycle + DAG |
| **1** | `bkg-mission` | `domains/thalassa/mission` | Mission→Milestone→Slice→Feature→Task |
| **1** | `bkg-scheduler` | `domains/thalassa/scheduler` | Deterministic DAG scheduler |
| **1** | `bkg-lanes` | `domains/styx/lanes` | Realm Bus IPC fabric |
| **1.5** | `bkg-ecs` | `domains/katoptron/ecs` | ECS — world model foundation |
| **1.5** | `bkg-projection` | `domains/katoptron/projection` | ProjectionCache + materializer |
| **1.5** | `bkg-identity` | `domains/speculum/identity` | Deterministic lineage + ancestry IDs |
| **2** | `bkg-secrets` | `domains/katoptron/secrets` | AES-256-GCM secrets + OS keychain |
| **2** | `bkg-approval` | `domains/katoptron/approval` | Approval gates + immutable audit |
| **2** | `bkg-capabilities` | `domains/speculum/capabilities` | Realm-scoped permissions + signed scopes |
| **2** | `bkg-eval` | `domains/katoptron/eval` | Task scorecards + evidence |
| **2** | `bkg-chat` | `domains/thalassa/chat` | Chat rooms + mailbox + SSE |
| **2** | `bkg-github` | `domains/thalassa/github` | Issue import + PR creation + OAuth |
| **2** | `bkg-plugins` | `domains/katoptron/plugins` | Plugin discovery + YAML manifest |
| **3** | `bkg-mesh` | `domains/arche/mesh` | Multi-node replication + leases |
| **3** | `bkg-vm` | `domains/thalassa/vm` | Tool sandbox VM + snapshots |
| **3** | `bkg-snapshot` | `domains/speculum/snapshot` | World snapshots — fork/export/restore |
| **3** | `bkg-migration` | `domains/speculum/migration` | Replay-safe schema migrations |
| **4** | `bkg-physics` | `domains/katoptron/physics` | DAG physics engine |
| **4** | `bkg-entropy` | `domains/katoptron/entropy` | Entropy + drift metrics + heat |
| **4** | `bkg-compiler` | `domains/katoptron/compiler` | Katoptron UI compiler pipeline |
| **4** | `bkg-render` | `domains/katoptron/render` | Render backend abstraction (ratatui/GPU/headless) |
| **4** | `bkg-diff` | `domains/speculum/diff` | Reality diff engine |
| **4** | `bkg-recovery` | `domains/speculum/recovery` | Crash reconstruction |
| **4** | `bkg-gc` | `domains/speculum/gc` | Causal GC: compaction, sealing, pruning |
| **4** | `bkg-lineage` | `domains/speculum/lineage` | Timeline ancestry graph |
| **4** | `bkg-simulation` | `domains/thalassa/simulation` | Deterministic execution simulator |
| **4** | `bkg-world` | `domains/katoptron/world` | Causal World Model — the true kernel |
| **4** | `bkg-operator` | `domains/anamnesis/operator` | Operator consciousness + intent |
| **5** | `bkg-consensus` | `domains/speculum/consensus` | Mesh arbitration (distributed) |

---

## Feature Detail by Crate

---

### Batch 0 — Critical Foundation (BEFORE everything else)

#### bkg-state 📋 CRITICAL
*The most important improvement in the entire architecture.*
```
reducer.rs          Reducer<E> trait — only canonical state mutator
                    apply(state: &RealmState, event: E) -> Result<RealmState>
realm_state.rs      RealmState — immutable, copy-on-write, structural sharing
projection.rs       Event → ReadModel (UI reads projections ONLY, never ledger)
transition.rs       Valid/invalid state transition classification
mutation.rs         Typed mutation record with causality trace
snapshot.rs         Frozen RealmState for archival + GC sealing
reconciliation.rs   Repair: partial writes, replay mismatches, corrupted capsules
invariants.rs       Compile-time + runtime invariant assertions
```
**Rule**: No crate may mutate state structs directly. Zero mutable globals. Immutable snapshots.

#### bkg-abi 📋 CRITICAL
*The nervous system. Without this: mesh breaks, plugins break, replay drifts, projections diverge.*
```
version.rs          AbiVersion, AbiCapability, AbiFeatureFlag, AbiCompatibility
envelope.rs         AbiEnvelope<T>: abi_version + payload_type + payload_hash + payload
event_abi.rs        Typed event serialization contract
packet_abi.rs       IPC packet format (bkg-lanes)
capsule_abi.rs      Capsule serialization contract
projection_abi.rs   Read-model wire format
bytecode_abi.rs     UI render instruction format
mesh_abi.rs         Cross-node replication format
plugin_abi.rs       Plugin contribution format
provider_abi.rs     LLM request/response normalization
```
**ABI Version Negotiation**: old mesh nodes, old plugins, old snapshots must remain readable.

#### bkg-clock 📋 CRITICAL
*No SystemTime::now() anywhere. Deterministic by construction.*
```
clock.rs            RealmClock — deterministic tick source
tick.rs             SequencedInstant = (realm_id, lamport_counter, wall_nanos_display_only)
vector_clock.rs     VectorClock — per-realm causality tracking
causal_time.rs      CausalTime — total causal ordering across realms
timeline.rs         Timeline — ordered tick sequence
epoch.rs            Epoch = genesis tick + current tick
divergence.rs       Equal lamport in same realm = determinism failure → halt
```

#### bkg-schema 📋 HIGH
*Without this: replay migrations, cross-version mesh, ABI upgrades become hell.*
```
registry.rs         EventSchemaRegistry — global schema catalog
schema.rs           EventSchema { id, version, producer_realm, reducer,
                                  projection_targets, causal_requirements,
                                  migration_strategy }
migration.rs        EventMigration — versioned payload transformers
compatibility.rs    Schema compatibility checks for mesh sync + replay
export.rs           Deterministic schema export for forensic replay
```

---

### Batch 1 — Core Application

#### bkg-project 📋 CRITICAL
```
registry.rs         ProjectRegistry — ~/.bkg/bkg-central.db (sled)
project.rs          Project: id, title, path, settings, created_at
settings.rs         ProjectSettings + GlobalSettings with 5 model lanes
isolation.rs        Strict project-scoped data isolation
path_map.rs         Per-node absolute path mappings (multi-node setups)
```

#### bkg-workflow 📋 HIGH
```
gate.rs             WorkflowGate: phase + verdict + retry config
verdict.rs          Verdict: APPROVE | REVISE | RETHINK | UNAVAILABLE
phase.rs            WorkflowPhase: Plan | PlanReview | Execute | ExecuteReview
graph.rs            ExecutionGraph: loops + retries + fallback branches +
                    parallel waves + conditional transitions
wave.rs             WaveExecution: parallel step sessions (no shared file scope)
engine.rs           WorkflowEngine: single gate transition orchestrator
fallback.rs         UNAVAILABLE → retry with fallback model + stricter instructions
```

#### bkg-query 📋 MEDIUM
*BQL — BKG Query Language. Without this: UI filters, telemetry, AI context = SQL/ECS chaos.*
```
ast.rs              BQL AST — SELECT/WHERE/ORDER BY/LIMIT
parser.rs           BQL parser (hand-written, deterministic)
executor.rs         BQL executor against ECS world + projection cache
planner.rs          Query plan optimization
types.rs            BQL types: string, number, enum, datetime, hash, ref
```
Example: `SELECT tasks WHERE status = "blocked" AND dependency.depth > 3 ORDER BY entropy DESC`

#### bkg-task 📋 CRITICAL
```
capsule/
  layout.rs         .bkg/tasks/{id}/ filesystem layout
  ledger.rs         Per-task event ledger (extends bkg-event)
  state.rs          TaskState reconstructed from events via bkg-state
  snapshot.rs       Frozen task state for archival
lifecycle/
  status.rs         TaskStatus: planning→todo→in-progress→review→done→archived
  transitions.rs    Valid transitions + guards + event emission
  stuck.rs          Stuck detection: loop detection + max retry budget
dag/
  graph.rs          DependencyGraph: DAG + cycle detection + topological sort
  overlap.rs        Shared file scope detection between parallel tasks
task.rs             Task: id, title, status, deps, capsule_path, prompt_md
search.rs           Full-text search across tasks + PROMPT.md content
```

#### bkg-mission 📋 HIGH
```
mission.rs          Mission: id + title + description + status
milestone.rs        Milestone: ordered set of slices
slice.rs            Slice: parallel feature set with shared scope partition
feature.rs          Feature: acceptance criteria + task_id + fix budget
autopilot.rs        MissionAutopilot: auto-activate next slice on completion
contracts.rs        MissionContract: assertions + success criteria
retry.rs            FixBudget: max retries for failed features
```

#### bkg-scheduler 📋 HIGH
```
scheduler.rs        TaskScheduler: priority queue + slot semaphore
dag.rs              DAG topological sort + cycle detection
priority.rs         Priority: Urgent > High > Normal > Low + FIFO tie-break
overlap.rs          OverlapGate: prevent tasks with shared file scope running concurrently
lease.rs            TaskLease: distributed lease with epoch fencing
blocker.rs          BlockerTracker: sticky blockers + unblock fanout
semaphore.rs        AgentSemaphore: concurrent agent slot limiting
```

#### bkg-lanes 📋 HIGH
```
realm_bus.rs        RealmBus: deterministic inter-realm transport
lane.rs             Lane: Critical | High | Normal | Background priority classes
packet.rs           BusPacket: signed + sequenced + replayable via bkg-abi
router.rs           LaneRouter: routes by (source_realm, target_realm, priority)
qos.rs              QosPolicy: latency guarantees per lane class
backpressure.rs     BackpressureController: congestion + flow control
```

---

### Batch 1.5 — ECS Foundation (MOVED UP — physics/compiler/world need this)

#### bkg-ecs 📋 CRITICAL
*Not Unity-style. Deterministic sparse archetype ECS.*
```
entity.rs           Entity: u64 id + generation counter (no random hashing)
component.rs        Component trait + typed storage
archetype.rs        Archetype: sparse-set column storage (stable iteration order)
world.rs            World: entity registry + component stores + system runner
query.rs            Query: filtered entity iteration (deterministic)
system.rs           System trait: deterministic update, replay-safe
allocation.rs       Replay-safe allocation: no non-deterministic ordering
```
**ALL DELPHOS entities become ECS entities:**
Task, Mission, Slice, Agent, Session, Approval, Message, WorkflowStep, Provider, MeshNode, Timeline, Capsule, Snapshot

#### bkg-projection 📋 HIGH
*UI reads ONLY projections. Never the ledger directly.*
*Projections are disposable, rebuildable, checksummed, invalidatable.*
```
materializer.rs     Event → ReadModel conversion (via bkg-state Reducer)
cache.rs            ProjectionCache: indexed, checksummed, invalidatable
index.rs            Fast entity lookup by type + attribute
invalidation.rs     Event-driven cache invalidation
subscription.rs     Push updates to live UI connections (SSE/WS)
rebuild.rs          Full projection rebuild from ledger (forensic mode)
```
**WARNING**: Projections must NEVER become the source of truth. If stale: rebuild from ledger.

#### bkg-identity 📋 HIGH
*Without this: timeline forking becomes uncontrollable.*
```
id.rs               DeterministicId::derive(seed, lineage, realm)
realm_id.rs         Realm identity + ancestry chain
mesh_id.rs          Mesh node identity + join lineage
capsule_id.rs       Capsule lineage (parent → child → fork)
timeline_id.rs      Timeline ancestry (origin → branch → fork)
operator_id.rs      Operator identity across sessions
```

---

### Batch 2 — Security + Features

#### bkg-secrets 📋 HIGH
```
store.rs            SecretsStore: AES-256-GCM, per-row nonce, SQLite
secret.rs           Secret: name + encrypted_value + scope + policy + provenance
scope.rs            SecretScope: Project(id) | Global
policy.rs           AccessPolicy: Auto | Prompt | Deny
keychain.rs         MasterKeyProvider: OS keychain + ~/.bkg/master.key
export.rs           SecretExport: materialize as env vars for worktree
provenance.rs       Track: who read each secret + when + which agent
sync.rs             Encrypted sync across mesh nodes
```

#### bkg-approval 📋 HIGH
```
gate.rs             ApprovalGate: pending → approved/denied → completed
audit.rs            Append-only audit trail (emits to bkg-event ledger)
dedup.rs            Dedupe by action context key
request.rs          ApprovalRequest: kind + description + context + risk
policy.rs           ActionPolicy: allow | block | require-approval
notification.rs     Task pause: pause_reason = "awaiting-approval"
```

#### bkg-capabilities 📋 HIGH
*Prevents agents from becoming "allmächtige Götter".*
```
capability.rs       Capability: realm-scoped permission token
grant.rs            CapabilityGrant: temporary, revocable, signed
scope.rs            ExecutionScope: signed scope for sandboxed execution
sandbox_policy.rs   Per-capability sandbox rules (integrates bkg-vm)
revocation.rs       Capability revocation + audit trail
```

#### bkg-eval 📋 MEDIUM
```
scorecard.rs        Scorecard: categories + weights + bands
evidence.rs         EvalEvidence: signals + AI scoring + suggestions
batch.rs            EvalBatch: scheduled evaluation on completed-task windows
persistence.rs      EvalResult + task snapshots for historical readability
followup.rs         FollowUp: normalized suggestions with suppression/dedup
```

#### bkg-chat 📋 MEDIUM
```
room.rs             ChatRoom: id + members + responder config
message.rs          ChatMessage: sender + content + attachments
mailbox.rs          Mailbox: per-user/agent inbox
streaming.rs        SSE streaming for real-time delivery
mention.rs          Mention routing: direct + ambient responders
attachment.rs       FileAttachment: metadata + upload path
```

#### bkg-github 📋 MEDIUM
```
auth.rs             GitHub OAuth + token management
import.rs           Issue → Task import with filtering
pr.rs               PR creation from task branches
badge.rs            Real-time PR/issue status badges
webhook.rs          GitHub webhook receiver for CI events
branch.rs           Branch push + squash/merge/rebase strategy
```

#### bkg-plugins 📋 MEDIUM
```
discovery.rs        GET /api/plugins/ui-slots, /dashboard-views
manifest.rs         YAML plugin manifest: dependencies + schema
loader.rs           Dynamic plugin module loading
store.rs            Global install metadata + per-project runtime state
contributions.rs    UI slots, dashboard views, runtime env vars, prompt injections
```

---

### Batch 3 — Infrastructure

#### bkg-mesh 📋 HIGH
```
node.rs             MeshNode: id + address + capabilities + health
discovery.rs        mDNS + central registry
lease.rs            MeshLease: epoch fencing + abandoned-lease recovery
sync.rs             State sync: task/mission/agent across nodes
write_queue.rs      WriteQueue: retryable writes for peer recovery
snapshot.rs         MeshSnapshot: topology checkpoint
consensus.rs        (stub → bkg-consensus in Batch 5)
```

#### bkg-vm 📋 HIGH
```
vm.rs               SandboxVm: deterministic execution environment
syscalls.rs         SyscallLayer: virtualized, replay-safe I/O
mounts.rs           VfsMount: scoped filesystem access
limits.rs           ResourceLimits: memory + CPU + time caps
snapshot.rs         VmSnapshot: rollback point (integrates bkg-snapshot)
process.rs          VmProcess: deterministic child process management
```

#### bkg-snapshot 📋 MEDIUM
```
world.rs            RealitySnapshot: full deterministic world state
realm.rs            RealmSnapshot: per-realm frozen state
timeline.rs         TimelineSnapshot: per-timeline frozen state
compression.rs      LZ4/zstd compression for snapshot storage
export.rs           Deterministic export (portable, versioned via bkg-abi)
restore.rs          Full restore + partial restore
fork.rs             Create new timeline from snapshot
```
CLI: `bkg snapshot create/fork/export/diff/restore`

#### bkg-migration 📋 MEDIUM
*Without this: old snapshots and replays break on schema changes.*
```
version.rs          SchemaVersion tracking per crate
strategy.rs         MigrationStrategy: lazy | eager | explicit
runner.rs           Migration runner: apply transformations to events
rollback.rs         Safe rollback to previous schema version
compatibility.rs    Forward + backward compatibility checks
```

---

### Batch 4 — Advanced Systems

#### bkg-physics 📋 HIGH
```
mass.rs             NodeMass: task complexity + dependency count
tension.rs          EdgeTension: dependency criticality
entropy.rs          EntropyState: system disorder measurement
simulation.rs       PhysicsSimulation: deterministic n-body layout
forces.rs           Force: gravity, spring, damping
stability.rs        StabilityThreshold: collapse detection + alert
```

#### bkg-entropy 📋 MEDIUM
*Telemetry physics — system properties as physical observables.*
```
entropy.rs          System disorder: unresolved dependencies / total nodes
pressure.rs         Agent load: active sessions / max capacity
heat.rs             Error rate × severity
stability.rs        Inverse of recent failure rate
latency.rs          Rolling P95 task completion time
drift.rs            Hash chain divergence rate
```

#### bkg-compiler 📋 HIGH
```
ast.rs              UiAst: abstract UI tree from realm state
compiler.rs         RealmState → UiAst → Bytecode
bytecode.rs         UiBytecode: portable render instructions (bkg-abi)
geometry.rs         GeometryGraph: layout from physics simulation
render_graph.rs     RenderGraph: dependency-aware render pass ordering
frame.rs            UiFrame: rendered output for one tick
```

#### bkg-render 📋 MEDIUM
*Backend abstraction for the compiler output.*
```
trait.rs            RenderBackend trait
ratatui/            Terminal: ratatui backend
ansi/               ANSI escape code backend
webgpu/             WebGPU backend (browser)
headless/           Headless: pixel buffer (CI + snapshot testing)
```

#### bkg-diff 📋 MEDIUM
```
state_diff.rs       RealmState before/after comparison
graph_diff.rs       DAG structural changes
capsule_diff.rs     Capsule version comparison
entropy_diff.rs     System disorder delta
timeline_diff.rs    Divergence between two replay paths
trace.rs            Causal mutation trace: who changed what, when, via which capability
```

#### bkg-recovery 📋 MEDIUM
```
crash.rs            Failure mode detection + classification
repair.rs           Partial replay repair from last checkpoint
capsule.rs          Corrupted capsule healing from event history
chain.rs            Event hash chain gap repair
rollback.rs         Rollback to last valid snapshot
```

#### bkg-gc 📋 HIGH (prevents "10 TB replay startup")
```
compaction.rs       Causal compaction: merge old events into checkpoints
sealing.rs          Snapshot sealing: mark timeline as immutable
freezing.rs         Timeline freezing: compress + archive
pruning.rs          Projection pruning: remove expired read models
compression.rs      Event segment compression (LZ4/zstd)
metrics.rs          GC pressure metrics (triggers at configurable thresholds)
```

#### bkg-lineage 📋 MEDIUM
```
timeline.rs         Timeline ancestry graph
fork.rs             Fork tracking: origin → branches
merge.rs            Merge history: which timelines were reconciled
ancestry.rs         Common ancestor lookup for diff
genealogy.rs        Full genealogy report for forensic replay
```

#### bkg-simulation 📋 MEDIUM
*For testing workflows and policies without running real agents.*
```
world.rs            SimulationWorld: deterministic fake world
agent.rs            SimAgent: scriptable fake agent
executor.rs         SimExecutor: runs workflow steps in simulation
replay.rs           Replay simulation against recorded events
oracle.rs           Oracle assertions for determinism verification
```

#### bkg-world 📋 ULTIMATE
*The true kernel. Where everything converges.*
```
graph.rs            RealityGraph: Entities + Relations + Causality
                    + Temporal flow + Entropy + Intent + Constraints
world.rs            World: full integration of ECS + physics + projections + BQL
query.rs            BQL execution engine (delegates to bkg-query)
causal.rs           Causal trace: who caused what, when, with which capability
intent.rs           Intent modeling: AI planning context extraction
physics.rs          World-level physics integration
projection.rs       World-level projection management
```

#### bkg-operator 📋 MEDIUM
```
intent.rs           OperatorIntent: inferred from action patterns
presence.rs         PresenceTracker: focus + attention model
attention.rs        AttentionMap: which realms/tasks are in focus
history.rs          InteractionHistory: causal interaction memory
adaptation.rs       AdaptiveOrchestration: modify behavior based on operator context
```

---

### Batch 5 — Consensus + Final

#### bkg-consensus 📋 LATER
*Distributed mesh arbitration. Raft-inspired but deterministic.*
```
raft.rs             Raft-inspired consensus (deterministic variant)
vote.rs             Vote collection + quorum detection
leader.rs           Leader election + epoch management
replication.rs      Log replication across mesh nodes
safety.rs           Safety invariants: no split-brain, no data loss
```

---

## Extended Capsule Lifecycle (extend bkg-capsule) 📋

```
Created     → object exists in registry
Mounted     → loaded into active memory
Active      → in use by a session/task/agent
Frozen      → read-only, sealed by hash
Forked      → new timeline created from this capsule
Archived    → moved to long-term storage + GC eligible
Corrupted   → hash mismatch detected by bkg-verifier
Recovered   → repaired by bkg-recovery from event history
```

---

## RealmDNA (extend bkg-kernel + cognition/realm-dna) 📋

*The "biological immutability" of each realm. Compile-time ontology firewall.*

```rust
struct RealmDNA {
    allowed_events:              Vec<EventSchemaId>,
    allowed_components:          Vec<ComponentTypeId>,
    allowed_capabilities:        Vec<CapabilityId>,
    allowed_lanes:               Vec<LaneClass>,
    allowed_reducers:            Vec<ReducerId>,
    allowed_projection_targets:  Vec<ProjectionId>,
    allowed_snapshot_scopes:     Vec<SnapshotScope>,
    allowed_tick_domains:        Vec<TickDomain>,
    allowed_physics_rules:       Vec<PhysicsRuleId>,
}
```

Without RealmDNA: Physics realm can mutate UI. Chat realm can corrupt Scheduler. Plugin realm breaks State.

---

## DomainEvent\<T\> (extend bkg-event) 📋

*Currently untyped `serde_json::Value` — must become typed for compile-time replay validation.*

```rust
struct DomainEvent<T: EventPayload> {
    id:              EventId,
    realm_id:        RealmId,
    schema_id:       EventSchemaId,
    schema_version:  SchemaVersion,
    timestamp:       SequencedInstant,   // bkg-clock — NO SystemTime
    producer:        ProducerId,
    causal_parent:   Option<EventId>,
    payload:         T,
    payload_hash:    Hash256,
    signature:       Option<Signature>,
}
```

---

## Summary Table

| Category | Crates Done | Crates Planned | Total |
|---|---|---|---|
| Cognition (system law) | 5 | 7 | 12 |
| Thalassa (execution) | 6 | 6 | 12 |
| Arche (persistence) | 2 | 1 | 3 |
| Styx (event provider) | 2 | 1 | 3 |
| Katoptron (observation) | 3 | 9 | 12 |
| Anamnesis (policy/memory) | 2 | 1 | 3 |
| Mnemos (replay) | 2 | 0 | 2 |
| Speculum (verification) | 0 | 9 | 9 |
| Reflection (UI) | 1 | extend | 1 |
| Threshold (CLI) | 1 | extend | 1 |
| **TOTAL** | **24** | **+34** | **58** |

Note: 52 was the earlier estimate; architectural analysis adds 6 more (bkg-schema, bkg-query, bkg-entropy, bkg-render, bkg-simulation, bkg-consensus).

---

*BKG v0.1.0 · 58 target crates · 258+ Fusion features · deterministic ontology engine*
*Single source of truth. One module, one location.*
