# BKG — Deterministic Ontology Engine

> **Single source of truth. One module, one location.**

[![Rust](https://img.shields.io/badge/rust-1.95+-orange)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-231_passing-green)](#testing)
[![Crates](https://img.shields.io/badge/crates-60+-blue)](#crate-map)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Architecture](https://img.shields.io/badge/architecture-DELPHOS-teal)](#architecture)

---

## What BKG is

BKG is not an agent framework. It is a **deterministic ontology engine**.

The AI agents are *inhabitants* of this world. Not the core of the world itself.

More precisely, BKG is a:

> **causally constrained, counterfactually aware semantic persistence system**

Every event is proven. Every rule is justified. Every state change is replayable.
The specification can evolve — but only within hard semantic invariants.

---

## Causal Data Flow

```
DomainEvent<T>  (typed, signed, causal_parent, schema_id)
    ↓ append
EventLedger     (append-only, BLAKE3 hash-chained, tamper-evident)
    ↓ apply() — StateTransitionFn<E> — the ONLY mutation path
RealmState      (immutable, copy-on-write, version-monotone)
    ↓ KernelMachine phase: ValidatingAbi → ... → Emitting
EventPipeline   (validate ABI → schema → clock → capability → causal → decide)
    ↓ ProjectionFactory::create() — KernelStamp required
ProjectionView<T>  (sealed, read-only, checksum-verified, rebuildable)
    ↓ BQL queries
Atlantean UI / Terminal / Headless CI
```

---

## The 9 Core Invariants

| # | Invariant | Enforcement |
|---|---|---|
| 1 | No mutation without event | `bkg-event` + `InvariantGuard` |
| 2 | No direct realm writes — EventPipeline only | `bkg-kernel` KernelMachine |
| 3 | State is reconstructed, never stored | `StateTransitionFn<E>` + `ReplaySession` |
| 4 | Replay must be identical | `ReplayIdentityProof` structural invariant |
| 5 | Drift = failure — detect and halt | `DriftDetector` + `kernel-alignment` |
| 6 | Tool execution is sealed | `bkg-vm` + `NoBypass<T>` |
| 7 | Causal ordering via lamport counter | `bkg-clock` + pipeline validation |
| 8 | One file = one responsibility | workspace convention |
| 9 | Single source of truth | one type, one crate |

---

## The Formal Kernel Stack (L0–L12)

The kernel (`bkg-kernel`) is a 12-layer formally verified system built over the last development cycle.

```
L0  ConstraintAlgebra          symbolic predicates over Q×Σ (pair-union, not cross-products)
L1  KernelMachine               deterministic δ, 18 phases × 29 inputs = 522 cells, TOTAL
L2  ProofCertificate            every transition produces a verifiable witness
L3  TraceSynthesizer            induces rules from observed execution
L4  DriftDetector               algebra ↔ kernel ↔ traces must stay synchronized
L5  SpecificationEntropy        Shannon + Gini + structural diversity + compression
L6  AlgebraStability            pinned anchors + synthesis cycle guard
L7  SemanticWeightLayer         necessity proof + causal importance + structural significance
L8  RuleSimplifier              safe transformation using semantic weights
L9  CounterfactualAnalyzer      BFS: "what path makes this rule matter?"
L10 SemanticFixationGuard       preserve rules reachable within N execution steps
L11 CounterfactualCompetitionLayer  unique_critical_cells breaks infinite preservation bias
L12 SemanticGrowthAnalyzer      expressiveness conservation: 80% semantic space stays free
```

See [`docs/KERNEL_FORMAL_SYSTEM.md`](docs/KERNEL_FORMAL_SYSTEM.md) for the complete specification.

---

## Architecture Overview

```
bkg/ (delphos/)
├── cognition/              System law and causality
│   ├── core                IDs, Hash256, BkgError, RealmId            ✅
│   ├── kernel              EventPipeline, KernelMachine, Realm         ✅ (L0–L12)
│   ├── event               TypedEvent<P>, DomainEvent, EventLedger     ✅
│   ├── state               RealmState, ProjectionView<T>, Reducer      ✅
│   ├── abi                 AbiEnvelope<T>, 7 typed ABIs                ✅
│   ├── clock               SequencedInstant, VectorClock               ✅
│   ├── enforce             Sealed, InvariantGuard, NoBypass<T>         ✅
│   ├── schema              EventSchemaRegistry, migration              ✅
│   ├── contracts           CausalContract                              ✅
│   ├── protocol            ACP JSON-RPC 2.0 + AgentBridge              ✅
│   ├── project             Project registry + settings                 ✅
│   ├── workflow            Plan→Review→Execute + ExecutionGraph        ✅
│   └── query               BQL engine                                  ✅
│
├── domains/
│   ├── thalassa/           Execution realm
│   │   ├── runtime         AgentRuntime (Telum sandbox)                ✅
│   │   ├── orchestrator    TaskGraph, EventBus, Scheduler              ✅
│   │   ├── providers       13 LLM providers (pi-free)                  ✅
│   │   ├── agents          7 agents + credentials (sandbox-agent)      ✅
│   │   ├── session         SessionManager + UniversalEvent             ✅
│   │   ├── exec            bash, file, grep, glob                      ✅
│   │   ├── task            Task capsules + lifecycle + DAG             ✅
│   │   ├── mission         Mission→Milestone→Slice→Task                ✅
│   │   ├── scheduler       Deterministic DAG scheduler                 ✅
│   │   ├── chat            Chat rooms + mailbox + SSE                  ✅
│   │   ├── github          Issue import + PR creation                  ✅
│   │   ├── simulation      Deterministic execution simulator           ✅
│   │   └── vm              Tool sandbox VM + snapshots                 ✅
│   ├── arche/              Persistence realm
│   │   ├── capsule         Capsule + lifecycle SM                      ✅
│   │   ├── store           sled + in-memory                            ✅
│   │   └── mesh            Multi-node replication + leases             ✅
│   ├── styx/               Event provider realm
│   │   ├── provider        SwdEngine                                   ✅
│   │   ├── tools           ledger_summary                              ✅
│   │   └── lanes           Realm Bus IPC fabric                        ✅
│   ├── katoptron/          Observation + projection realm
│   │   ├── crypto          BLAKE3, Ed25519                             ✅
│   │   ├── verifier        hash-chain + PermissionEnforcer             ✅
│   │   ├── telemetry       model call tracking + entropy               ✅
│   │   ├── approval        Approval gates + audit                      ✅
│   │   ├── secrets         AES-256-GCM + OS keychain                  ✅
│   │   ├── eval            Scorecards + evidence                       ✅
│   │   ├── plugins         Plugin discovery + manifest                 ✅
│   │   ├── ecs             Deterministic sparse-archetype ECS          ✅
│   │   ├── projection      ProjectionCache + MaterializerKernel        ✅
│   │   ├── physics         DAG physics engine (n-body layout)          ✅
│   │   ├── compiler        Katoptron UI compiler pipeline              ✅
│   │   ├── entropy         Entropy + drift metrics                     ✅
│   │   ├── render          Render backends (ANSI, headless)            ✅
│   │   └── world           Causal World Model — the true kernel        ✅
│   ├── anamnesis/          Policy + memory realm
│   │   ├── policy          PolicyEngine                                ✅
│   │   ├── memory          MemoryGraph (causal graph)                  ✅
│   │   └── operator        Operator consciousness + intent             ✅
│   ├── mnemos/             Replay realm
│   │   ├── memory          MemoryGraph                                 ✅
│   │   └── replay          ReplayEngine + divergence                   ✅
│   └── speculum/           Verification + audit realm
│       ├── capabilities    Realm permissions + scopes                  ✅
│       ├── snapshot        World snapshots (fork/restore)              ✅
│       ├── diff            Reality diff engine                         ✅
│       ├── recovery        Crash reconstruction                        ✅
│       ├── gc              Causal garbage collection                   ✅
│       ├── identity        Deterministic lineage                       ✅
│       ├── lineage         Timeline ancestry graph                     ✅
│       └── migration       Replay-safe schema migrations               ✅
│
├── reflection/
│   └── ui/atlantean        Cyberpunk/Atlantis dashboard                ✅
└── threshold/
    └── cli                 `bkg` binary                               ✅
```

---

## Quick Start

```bash
# Clone + build
git clone https://github.com/soniaschuh88-ship-it/bkg.git && cd bkg
cargo build --workspace

# Run all tests (231 tests across 60+ crates)
cargo test --workspace

# Quality checks
cargo clippy --workspace -- -D warnings

# Start the Atlantean dashboard (http://localhost:7878)
cargo run -p bkg-atlantean
```

---

## Integrated Systems

| Source | What was integrated | Where |
|---|---|---|
| [pi-free](https://github.com/apmantza/pi-free) | 13 LLM providers, free detection, toggles, telemetry | `bkg-providers`, `bkg-telemetry` |
| [sandbox-agent](https://github.com/rivet-dev/sandbox-agent) | 7 agents, universal events, ACP protocol, Inspector | `bkg-agents`, `bkg-session`, `bkg-acp` |
| [Fusion](https://github.com/fusion-ai/fusion) | 258+ features (task, mission, workflow, mesh, eval…) | all of `domains/` |

---

## Testing

```
Total tests: 231 passing
Clippy:      0 warnings with -D warnings
Stubs:       0 (zero allow(dead_code) files remaining)
```

Key test suites:
- `bkg-kernel`: 231 tests covering L0–L12 formal stack
- `bkg-state`: 33 tests covering projection isolation, contracts, rebuild guarantees
- `bkg-event`: 20 tests covering TypedEvent<P> + 9 canonical event types
- `bkg-enforce`: 15 tests covering sealed traits, invariant guards
- All domain crates: full unit test coverage

---

## Contributing

1. **Single source of truth** — every concept lives in exactly one crate
2. **No direct state mutation** — only via `StateTransitionFn<E>::apply()`
3. **No `SystemTime::now()`** — use `bkg-clock` `SequencedInstant`
4. **Replay-safe** — every operation must produce identical output on replay
5. **Projection-only UI** — UI reads from `ProjectionView<T>` only, never `RealmState`
6. **EventPipeline required** — all events must pass through `EventPipeline.process()`

```bash
cargo test --workspace && cargo clippy --workspace -- -D warnings
```

---

*BKG v0.1.0 · DELPHOS · Deterministic Ontology Engine*  
*231 tests · 60+ crates · 12-layer formal kernel · Single source of truth*
