# BKG — Deterministic Multi-Realm Execution System

> **Single source of truth. One module, one location.**
> *BKG is not a task manager. It is a deterministic OS for causal agent orchestration.*

[![Rust](https://img.shields.io/badge/rust-1.95+-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Architecture](https://img.shields.io/badge/architecture-DELPHOS-teal)](#architecture)

---

## What BKG is

```
Event Ledger  →  Reducer  →  World Graph  →  Physics  →  Compiler  →  Projection  →  UI
```

Every state in BKG emerges from events. No hidden mutations. No side-effects outside the ledger.
The UI is the visible shadow of event causality — nothing more, nothing less.

**BKG unifies:**
- [pi-free](https://github.com/apmantza/pi-free) — 13 LLM providers, free-model detection, per-provider toggles
- [sandbox-agent](https://github.com/rivet-dev/sandbox-agent) — multi-agent runtime, universal event schema, ACP protocol
- [Fusion](https://github.com/fusion-ai/fusion) — deterministic task orchestration, mission hierarchy, workflow gates
- **DELPHOS** — the causal world model that binds them all

---

## The 9 Invariants

These can never be violated. Enforced at compile time where possible.

| # | Invariant | Crate |
|---|---|---|
| 1 | No mutation without event | `bkg-event` (append-only) |
| 2 | No direct realm writes — only via Contracts | `bkg-kernel` RealmRouter |
| 3 | No state outside the ledger — reconstruct, don't store | `bkg-swd` + `bkg-capsule` |
| 4 | Replay must be identical — same seed + same ledger = same output | `bkg-replay` + BLAKE3 |
| 5 | Drift = failure — detect and halt | `bkg-verifier` + `bkg-diff` |
| 6 | VM isolation — tool execution is sealed | `bkg-vm` |
| 7 | Causal ordering — vector clocks per realm | `bkg-clock` |
| 8 | One file = one responsibility | workspace convention |
| 9 | Single source of truth — no type defined in more than one crate | workspace structure |

---

## Architecture — DELPHOS

```
bkg/
└── delphos/
    ├── cognition/              ← System law and causality
    │   ├── core/               typed IDs, Hash256, BkgError, ExecutionSeed
    │   ├── kernel/             Genesis, RealmRouter, ContractValidator, Arbitrator
    │   ├── event/              Event, EventLedger, DomainEvent<T>
    │   ├── contracts/          CausalContract — cross-realm only
    │   ├── protocol/           ACP JSON-RPC 2.0, AgentBridge, InferenceProxy
    │   ├── state/              RealmStateMachine, reducer, projections, snapshots ★
    │   ├── abi/                Universal Realm ABI — events, packets, projections ★
    │   ├── clock/              Realm Clock, vector clocks, causal ordering ★
    │   ├── project/            Project registry, settings, isolation ★
    │   └── workflow/           Plan→Review→Execute gates, verdicts, ExecutionGraph ★
    │
    └── domains/
        ├── thalassa/           ← Execution realm
        │   ├── runtime/        AgentRuntime (Telum sandbox)
        │   ├── orchestrator/   TaskGraph, EventBus, Scheduler
        │   ├── providers/      13 LLM providers (pi-free)
        │   ├── agents/         7 agents, credentials, status (sandbox-agent)
        │   ├── session/        SessionManager, UniversalEvent, SSE (sandbox-agent)
        │   ├── exec/           bash, file, grep, glob tools
        │   ├── task/           Task capsules, lifecycle, DAG dependencies ★
        │   ├── mission/        Mission→Milestone→Slice→Feature→Task ★
        │   ├── scheduler/      Deterministic DAG, priority queue, overlap gating ★
        │   ├── chat/           Chat rooms, mailbox, streaming SSE ★
        │   ├── github/         Issue import, PR creation, OAuth ★
        │   └── vm/             Tool execution sandbox VM ★
        ├── arche/              ← Persistence realm
        │   ├── capsule/        Capsule + CapsuleManager (lifecycle state machine)
        │   ├── store/          sled + in-memory backends
        │   └── mesh/           Multi-node replication, lease management ★
        ├── styx/               ← Event provider realm
        │   ├── provider/       SwdEngine — audit protocol
        │   ├── tools/          ledger_summary, dump_realm
        │   └── lanes/          Realm Bus / IPC fabric ★
        ├── katoptron/          ← Observation and projection realm
        │   ├── crypto/         BLAKE3, Ed25519, key derivation
        │   ├── verifier/       hash-chain, PermissionEnforcer
        │   ├── telemetry/      model call tracking, quota monitor
        │   ├── approval/       Approval gates, immutable audit trail ★
        │   ├── secrets/        AES-256-GCM secrets, OS keychain ★
        │   ├── eval/           Task scorecards, evidence ★
        │   ├── plugins/        Plugin discovery, manifest loader ★
        │   ├── ecs/            Entity-Component-System — foundational world model ★
        │   ├── projection/     ProjectionCache, materializer, realtime subscriptions ★
        │   ├── physics/        DAG physics: mass, tension, entropy ★
        │   ├── compiler/       Katoptron UI compiler pipeline ★
        │   └── world/          Causal World Model — the ultimate integration ★
        ├── anamnesis/          ← Policy and memory realm
        │   ├── policy/         PolicyEngine + built-in policies
        │   ├── memory/         MemoryGraph (causal graph, decay, dreams)
        │   └── operator/       Operator consciousness, intent tracking ★
        ├── mnemos/             ← Replay and reconstruction realm
        │   ├── memory/         MemoryGraph
        │   └── replay/         ReplayEngine, DivergenceDetector
        └── speculum/           ← Audit and verification realm
            ├── capabilities/   Realm permissions, signed scopes ★
            ├── snapshot/       Global world snapshots (fork, export, restore) ★
            ├── diff/           Reality diff engine ★
            └── recovery/       Crash reconstruction, partial replay repair ★

    └── reflection/
        └── ui/
            └── atlantean/      Cyberpunk/Atlantis dashboard (Axum + embedded UI)

    └── threshold/
        └── cli/                `bkg` binary — all commands
```

★ = planned / in progress

---

## Quick Start

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone
git clone https://github.com/soniaschuh88-ship-it/bkg.git
cd bkg

# Build everything
cargo build --workspace

# Initialize a project
cargo run -p bkg-cli -- init

# Start the dashboard (http://localhost:7878)
cargo run -p bkg-atlantean

# Run all tests
cargo test --workspace

# Check code quality
cargo clippy --workspace -- -D warnings
```

---

## Dashboard

Start the cyberpunk/Atlantis web dashboard:

```bash
cargo run -p bkg-atlantean
# → http://localhost:7878
```

**Pages:**
- **Chat** — LLM conversation (Private: WebLLM/Ollama, Cloud: 13 free providers)
- **Providers** — All 13 providers with tier, toggle, signup links
- **Agents** — 7 agents (Claude, Codex, OpenCode, Amp, Pi, Cursor, Mock)
- **Inspector** — Live session browser + SSE event stream
- **Dashboard** — Stats, telemetry, provider status
- **Admin** — Global provider keys, default model, free-only toggle

---

## CLI Reference

```bash
# Core
bkg init                          # Genesis + Styx ledger
bkg run --input '...'             # SWD-audited execution
bkg verify                        # Hash-chain verification
bkg replay                        # Reconstruct state from ledger
bkg status                        # System state as JSON
bkg chat [--prompt "..."]         # LLM conversation

# Agents (sandbox-agent)
bkg agent list
bkg agent spawn --name X --mode bkg_supervised

# Providers (pi-free)
bkg providers list
bkg providers models <id>
bkg providers toggle <id>
bkg providers refresh <id|all>

# Sessions (sandbox-agent inspector)
bkg session list
bkg session create --agent claude
bkg session send <id> "..."
```

---

## Provider Fallback Chain

The same chain is used everywhere — providers, agents, secrets:

```
1. User's own key  (BKG user config)
2. Admin global    (~/.bkg/global-providers.json)
3. Env variable    (ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.)
4. Anonymous       (Kilo + LLM7 — no key required)
```

---

## Private vs Cloud Mode

**Private mode** (WebLLM + Ollama):
- Models run in-browser via WebGPU (zero data leaves the machine)
- Ollama proxied through `/tunnel/ollama/*` (no CORS issues)

**Cloud mode** (13 free providers):
- Anthropic, OpenRouter, SambaNova, LLM7, Kilo, Cline + 7 more
- Free-only filter enforced by default
- Per-provider toggle: `bkg providers toggle <id>`

---

## Contributing

This project follows the DELPHOS architecture principles:

1. **Single source of truth** — every concept lives in exactly one crate
2. **One module, one location** — no duplication across crates
3. **Event-first** — all state mutations emit events to the ledger
4. **Replay-safe** — no `SystemTime::now()` in business logic (use `bkg-clock`)
5. **No hidden state** — reconstruct from events, never store derived state

```bash
# Before submitting a PR
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## License

MIT — see [LICENSE](LICENSE)

---

*BKG v0.1.0 · DELPHOS architecture · Single source of truth. One module, one location.*
