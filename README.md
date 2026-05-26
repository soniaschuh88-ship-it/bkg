# BKG — Deterministic Ontology Engine

> **Single source of truth. One module, one location.**

[![Rust](https://img.shields.io/badge/rust-1.95+-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Architecture](https://img.shields.io/badge/architecture-DELPHOS-teal)](#architecture)

---

## What BKG is

BKG is not an agent framework. It is a **deterministic ontology engine**.

The AI agents are *inhabitants* of this world. Not the core of the world itself.

```
Event Ledger
    ↓ Reducer<E> — only canonical state mutator
Realm State (immutable, copy-on-write)
    ↓ ECS World sync
World Graph (Entities + Relations + Causality)
    ↓ Physics simulation
Geometry
    ↓ Katoptron UI Compiler
Render Bytecode
    ↓ Projection Cache (indexed, checksummed, rebuildable)
Atlantean UI / Terminal
```

This is not "AI coding tool". This is a **deterministic causal operating substrate** with replayable world simulation for agentic systems.

---

## Not

- LangGraph / AutoGen / CrewAI / Claude Code clone
- Agent framework with task board
- Event-driven microservices

## But

- Event-sourced causal operating substrate
- Deterministic timeline engine
- Replayable world simulation
- Causal ontology reducer
- Projection compilation system
- Timeline forking + ancestry tracking
- Execution physics layer

---

## The 9 Invariants

| # | Invariant | Crate |
|---|---|---|
| 1 | No mutation without event | `bkg-event` |
| 2 | No direct realm writes — Contracts only | `bkg-kernel` RealmRouter |
| 3 | State is reconstructed, never stored | `bkg-state` Reducer |
| 4 | Replay must be identical | `bkg-clock` + BLAKE3 chain |
| 5 | Drift = failure — detect and halt | `bkg-verifier` + `bkg-diff` |
| 6 | Tool execution is sealed | `bkg-vm` |
| 7 | Causal ordering via vector clocks | `bkg-clock` |
| 8 | One file = one responsibility | workspace convention |
| 9 | Single source of truth | one type, one crate |

### The Reducer Rule (strongest invariant)

```rust
pub trait Reducer<E> {
    fn apply(state: &RealmState, event: E) -> Result<RealmState>;
}
```

**No crate may mutate state structs directly.**
Immutable snapshots. Structural sharing. Copy-on-write. Zero mutable globals.

---

## Architecture Overview

```
bkg/ (delphos/)
├── cognition/          System law and causality
│   ├── core            IDs, Hash256, BkgError               ✅
│   ├── kernel          Genesis, RealmRouter, Arbitrator      ✅+
│   ├── event           Event, Ledger, DomainEvent<T>         ✅+
│   ├── contracts       CausalContract (cross-realm only)     ✅
│   ├── protocol        ACP JSON-RPC 2.0 + AgentBridge        ✅
│   ├── state           RealmStateMachine, Reducer            📋
│   ├── abi             Universal Realm ABI + versioning      📋
│   ├── clock           Vector clocks, causal ordering        📋
│   ├── project         Project registry + settings           📋
│   ├── workflow        Plan→Review→Execute gates             📋
│   ├── schema          EventSchemaRegistry                   📋
│   └── query           BQL engine                           📋
│
├── domains/
│   ├── thalassa/       Execution realm
│   │   ├── runtime     AgentRuntime (Telum sandbox)          ✅
│   │   ├── orchestrator TaskGraph, EventBus, Scheduler       ✅
│   │   ├── providers   13 LLM providers (pi-free)            ✅
│   │   ├── agents      7 agents + credentials (sandbox-agent) ✅
│   │   ├── session     SessionManager + UniversalEvent       ✅
│   │   ├── exec        bash, file, grep, glob                ✅
│   │   ├── task        Task capsules + lifecycle             📋
│   │   ├── mission     Mission→Milestone→Slice→Task          📋
│   │   ├── scheduler   Deterministic DAG scheduler           📋
│   │   ├── chat        Chat rooms + mailbox                  📋
│   │   ├── github      Issue import + PR creation            📋
│   │   ├── simulation  Deterministic execution simulator     📋
│   │   └── vm          Tool sandbox VM                       📋
│   ├── arche/          Persistence realm
│   │   ├── capsule     Capsule + lifecycle SM                ✅+
│   │   ├── store       sled + in-memory                      ✅
│   │   └── mesh        Multi-node replication                📋
│   ├── styx/           Event provider realm
│   │   ├── provider    SwdEngine                             ✅
│   │   ├── tools       ledger_summary                        ✅
│   │   └── lanes       Realm Bus IPC fabric                  📋
│   ├── katoptron/      Observation + projection realm
│   │   ├── crypto      BLAKE3, Ed25519                       ✅
│   │   ├── verifier    hash-chain + PermissionEnforcer       ✅
│   │   ├── telemetry   model call tracking + entropy         ✅+
│   │   ├── approval    Approval gates + audit                📋
│   │   ├── secrets     AES-256-GCM + OS keychain             📋
│   │   ├── eval        Scorecards + evidence                 📋
│   │   ├── plugins     Plugin discovery + manifest           📋
│   │   ├── ecs         Entity-Component-System (foundation)  📋
│   │   ├── projection  ProjectionCache + materializer        📋
│   │   ├── physics     DAG physics engine                    📋
│   │   ├── compiler    Katoptron UI compiler pipeline        📋
│   │   ├── entropy     Entropy + drift metrics               📋
│   │   └── world       Causal World Model (the true kernel)  📋
│   ├── anamnesis/      Policy + memory realm
│   │   ├── policy      PolicyEngine                          ✅
│   │   ├── memory      MemoryGraph (causal graph)            ✅
│   │   └── operator    Operator consciousness                📋
│   ├── mnemos/         Replay realm
│   │   ├── memory      MemoryGraph                           ✅
│   │   └── replay      ReplayEngine + divergence             ✅
│   └── speculum/       Verification + audit realm
│       ├── capabilities Realm permissions + scopes           📋
│       ├── snapshot    World snapshots (fork/restore)         📋
│       ├── diff        Reality diff engine                    📋
│       ├── recovery    Crash reconstruction                   📋
│       ├── gc          Causal garbage collection             📋
│       ├── identity    Deterministic lineage                 📋
│       ├── lineage     Timeline ancestry graph               📋
│       ├── migration   Replay-safe schema migrations         📋
│       └── consensus   Mesh arbitration                      📋
│
├── reflection/
│   └── ui/atlantean    Cyberpunk/Atlantis dashboard          ✅+
└── threshold/
    └── cli             `bkg` binary                          ✅+
```

---

## Quick Start

```bash
# Clone + build
git clone https://github.com/soniaschuh88-ship-it/bkg.git && cd bkg
cargo build --workspace

# Initialize a BKG project
cargo run -p bkg-cli -- init

# Start the Atlantean dashboard (http://localhost:7878)
cargo run -p bkg-atlantean

# Run all tests + quality checks
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Provider Fallback Chain

```
1. User's own key    (BKG user config per project)
2. Admin global key  (~/.bkg/global-providers.json)
3. Env variable      (ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.)
4. Anonymous tier    (Kilo + LLM7 — no key required)
```

---

## Private vs Cloud Mode

**Private** — zero data leaves the machine:
- WebLLM in-browser (WebGPU + CDN-loaded models)
- Ollama tunnel via `/tunnel/ollama/*`

**Cloud** — 13 free providers:
- Anthropic, OpenRouter, SambaNova, LLM7, Kilo, Cline, + 7 more
- Free-only filter enforced by default, per-provider toggle persisted

---

## Integrated Systems

| Source | What was ported | Where |
|---|---|---|
| [pi-free](https://github.com/apmantza/pi-free) | 13 LLM providers, free detection, toggles, telemetry | `bkg-providers`, `bkg-telemetry` |
| [sandbox-agent](https://github.com/rivet-dev/sandbox-agent) | 7 agents, universal events, ACP protocol, Inspector | `bkg-agents`, `bkg-session`, `bkg-acp` |
| [Fusion](https://github.com/fusion-ai/fusion) | 258+ features (task, mission, workflow, mesh, eval…) | all of `domains/thalassa/*` + more |

---

## Contributing

1. **Single source of truth** — every concept lives in exactly one crate
2. **No direct state mutation** — only via `Reducer<E>::apply()`
3. **No `SystemTime::now()`** — use `bkg-clock` `SequencedInstant`
4. **Replay-safe** — every operation must produce identical output on replay
5. **Projection-only UI** — UI reads from `bkg-projection` only, never the ledger

```bash
cargo test --workspace && cargo clippy --workspace -- -D warnings
```

---

*BKG v0.1.0 · DELPHOS · Deterministic Ontology Engine*
*Single source of truth. One module, one location.*
