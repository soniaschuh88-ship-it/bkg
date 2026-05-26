# BKG — Features (current state)

> **Single source of truth. One module, one location.**

---

## Status: ✅ All Batches 0–4 complete. Zero stubs.

---

## Crate Map (60+ crates, all implemented)

### Cognition — System Law

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-core` | `cognition/core` | ✅ | RealmId, Hash256, BkgError, BkgResult |
| `bkg-kernel` | `cognition/kernel` | ✅ L0–L12 | EventPipeline, KernelMachine, Realm, 25 modules |
| `bkg-event` | `cognition/event` | ✅ | TypedEvent<P>, DomainEvent, 9 canonical types |
| `bkg-state` | `cognition/state` | ✅ | RealmState, ProjectionView<T>, MaterializerKernel |
| `bkg-abi` | `cognition/abi` | ✅ | AbiEnvelope<T>, 7 typed ABI modules |
| `bkg-clock` | `cognition/clock` | ✅ | SequencedInstant, VectorClock |
| `bkg-enforce` | `cognition/enforce` | ✅ | Sealed, InvariantGuard, NoBypass<T>, WorkspaceLints |
| `bkg-schema` | `cognition/schema` | ✅ | EventSchemaRegistry, MigrationStrategy |
| `bkg-contracts` | `cognition/contracts` | ✅ | CausalContract |
| `bkg-protocol` | `cognition/protocol` | ✅ | ACP JSON-RPC 2.0, AgentBridge |
| `bkg-project` | `cognition/project` | ✅ | ProjectRegistry, ProjectSettings |
| `bkg-workflow` | `cognition/workflow` | ✅ | WorkflowGate, ExecutionGraph, Verdict |
| `bkg-query` | `cognition/query` | ✅ | BQL parser, AST, executor, planner |

### Thalassa — Execution Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-runtime` | `thalassa/runtime` | ✅ | AgentRuntime, Telum sandbox |
| `bkg-orchestrator` | `thalassa/orchestrator` | ✅ | TaskGraph, EventBus, Scheduler |
| `bkg-providers` | `thalassa/providers` | ✅ | 13 LLM providers, free detection |
| `bkg-agents` | `thalassa/agents` | ✅ | 7 agents, BkgSupervised mode |
| `bkg-session` | `thalassa/session` | ✅ | UniversalEvent (10 types), BkgSession |
| `bkg-exec` | `thalassa/exec` | ✅ | bash, file, grep, glob executors |
| `bkg-task` | `thalassa/task` | ✅ | Task capsules, lifecycle SM, DAG |
| `bkg-mission` | `thalassa/mission` | ✅ | Mission→Milestone→Slice→Task |
| `bkg-scheduler` | `thalassa/scheduler` | ✅ | DAG scheduler, OverlapGate |
| `bkg-chat` | `thalassa/chat` | ✅ | ChatRoom, Mailbox, mentions |
| `bkg-github` | `thalassa/github` | ✅ | Issue import, PR creation |
| `bkg-simulation` | `thalassa/simulation` | ✅ | SimWorld, SimAgent, Oracle |
| `bkg-vm` | `thalassa/vm` | ✅ | SandboxVm, SyscallFilter, ResourceLimits |

### Arche — Persistence Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-capsule` | `arche/capsule` | ✅ | Capsule, lifecycle SM |
| `bkg-store` | `arche/store` | ✅ | sled + in-memory |
| `bkg-mesh` | `arche/mesh` | ✅ | MeshNode, LeaseRegistry, SyncRecord |

### Styx — Event Provider Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-swd` | `styx/provider` | ✅ | SwdEngine |
| `bkg-tools` | `styx/tools` | ✅ | ledger_summary |
| `bkg-lanes` | `styx/lanes` | ✅ | RealmBus, LaneClass, BusPacket, LaneRouter |

### Katoptron — Observation + Projection Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-crypto` | `katoptron/crypto` | ✅ | BLAKE3, Ed25519 |
| `bkg-verifier` | `katoptron/verifier` | ✅ | hash-chain, PermissionEnforcer |
| `bkg-telemetry` | `katoptron/telemetry` | ✅ | model call tracking, quota |
| `bkg-approval` | `katoptron/approval` | ✅ | ApprovalGate, ApprovalAudit |
| `bkg-secrets` | `katoptron/secrets` | ✅ | SecretsStore, AES-256-GCM |
| `bkg-eval` | `katoptron/eval` | ✅ | Scorecard (A-F bands), EvalBatch |
| `bkg-plugins` | `katoptron/plugins` | ✅ | PluginRegistry, PluginLoader |
| `bkg-ecs` | `katoptron/ecs` | ✅ | World (BTreeMap stable order), Query |
| `bkg-projection` | `katoptron/projection` | ✅ | ProjectionCache, Materializer |
| `bkg-physics` | `katoptron/physics` | ✅ | PhysicsSimulation, system_entropy |
| `bkg-compiler` | `katoptron/compiler` | ✅ | UiAst, UiCompiler (deterministic) |
| `bkg-entropy` | `katoptron/entropy` | ✅ | MetricSnapshot, SystemMetrics |
| `bkg-render` | `katoptron/render` | ✅ | HeadlessBackend, ANSI renderer |
| `bkg-world` | `katoptron/world` | ✅ | WorldGraph, CausalChain, BQL bridge |

### Anamnesis — Policy + Memory Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-policy` | `anamnesis/policy` | ✅ | PolicyEngine |
| `bkg-memory` | `mnemos/memory` | ✅ | MemoryGraph |
| `bkg-operator` | `anamnesis/operator` | ✅ | OperatorIntent, AttentionMap |

### Mnemos — Replay Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-replay` | `mnemos/replay` | ✅ | ReplayEngine, divergence detection |

### Speculum — Verification + Audit Realm

| Crate | Path | Status | Key Types |
|---|---|---|---|
| `bkg-capabilities` | `speculum/capabilities` | ✅ | CapabilitySet, CapabilityGrant |
| `bkg-snapshot` | `speculum/snapshot` | ✅ | RealitySnapshot (fork, gc_eligible) |
| `bkg-diff` | `speculum/diff` | ✅ | StateDiff, CausalTrace |
| `bkg-recovery` | `speculum/recovery` | ✅ | CrashClassification, RepairStrategy |
| `bkg-gc` | `speculum/gc` | ✅ | GcPolicy, GcPressure, GcRun |
| `bkg-identity` | `speculum/identity` | ✅ | DeterministicId, AncestryChain |
| `bkg-lineage` | `speculum/lineage` | ✅ | LineageGraph, ForkRecord |
| `bkg-migration` | `speculum/migration` | ✅ | VersionMap, MigrationRunner |

### UI + CLI

| Crate | Path | Status |
|---|---|---|
| `bkg-atlantean` | `reflection/ui/atlantean` | ✅ Cyberpunk dashboard |
| `bkg-cli` | `threshold/cli` | ✅ `bkg` binary |

---

## Kernel Formal System Features (bkg-kernel)

The kernel alone contains 25 modules implementing L0–L12. See `docs/KERNEL_FORMAL_SYSTEM.md`.

Key capabilities:
- **Proof-carrying execution**: every state change has a verifiable certificate
- **Self-healing specification**: rules are induced from observed behavior
- **Drift detection**: algebra ↔ kernel ↔ traces must stay synchronized
- **Semantic weights**: rules are scored by necessity + causal importance
- **Counterfactual reasoning**: "what would need to happen for this rule to matter?"
- **Expressiveness conservation**: 80.1% of semantic space stays free for growth

---

## Planned

| Crate | Priority | Notes |
|---|---|---|
| `bkg-consensus` | LOW | Raft-inspired mesh arbitration |

---

*BKG v0.1.0 · 60+ crates implemented · Zero stubs*
