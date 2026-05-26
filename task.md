# BKG — task.md

> **System ethic**: *Single source of truth. One module, one location.*

---

## System Definition

**BKG v0.1.0** is a deterministic, replayable multi-realm execution system:

- Every state emerges from events
- Every event is fully reconstructable
- Every deviation creates a new timeline
- **SWD** (System Write Descriptor) is the primary audit protocol
- **Capsules** are isolated, versioned state containers

---

## Architecture — DELPHOS Layout

```
bkg/
└── delphos/
    ├── threshold/cli/          bkg-cli        — all CLI commands
    ├── cognition/
    │   ├── core/               bkg-core       — typed IDs, Hash256, BkgError
    │   ├── kernel/             bkg-kernel     — Genesis lock, RealmRouter
    │   ├── event/              bkg-event      — Event, EventLedger (memory+file)
    │   └── contracts/          bkg-contracts  — CausalContract (cross-realm only)
    └── domains/
        ├── thalassa/
        │   ├── runtime/        bkg-runtime    — AgentRuntime (Telum sandbox)
        │   └── orchestrator/   bkg-orchestrator — TaskGraph, EventBus, Scheduler
        ├── arche/
        │   ├── capsule/        bkg-capsule    — immutable versioned state containers
        │   └── store/          bkg-store      — sled + in-memory backends
        ├── styx/
        │   ├── provider/       bkg-swd        — SwdEngine lifecycle
        │   └── tools/          bkg-tools      — ledger inspection
        ├── katoptron/
        │   ├── crypto/         bkg-crypto     — BLAKE3, Ed25519
        │   └── verifier/       bkg-verifier   — hash-chain, PermissionEnforcer ★
        ├── anamnesis/
        │   └── policy/         bkg-policy     — PolicyEngine
        └── mnemos/
            ├── memory/         bkg-memory     — weighted causal graph
            └── replay/         bkg-replay     — ReplayEngine, divergence detection
```

---

## Crates (18)

| Crate | Responsibility |
|---|---|
| `bkg-core` | `RealmId`, typed IDs, `Hash256`, `ExecutionSeed`, `BkgError` |
| `bkg-crypto` | BLAKE3 hashing, Ed25519 sign/verify, seed derivation |
| `bkg-event` | `Event`, hash-chained `EventLedger`, `LaneEvent` types |
| `bkg-contracts` | `CausalContract` — the only legal cross-realm message |
| `bkg-kernel` | `Genesis` lock, `RealmRouter`, `CausalContractValidator` |
| `bkg-swd` | `SwdEngine` — init → capture → commit → verify → archive |
| `bkg-capsule` | `Capsule` + `CapsuleManager` — immutable history, versioning |
| `bkg-store` | `InMemoryStore` + `SledStore` — capsule persistence |
| `bkg-memory` | `MemoryGraph` — importance = impact × recurrence × depth |
| `bkg-replay` | `ReplayEngine`, `DivergenceDetector`, `BranchReport` |
| `bkg-verifier` | hash-chain, capsule integrity, drift, **`PermissionEnforcer`** |
| `bkg-policy` | `PolicyEngine` + built-in event policies |
| `bkg-runtime` | `AgentRuntime` — Telum sandbox, SWD-recorded task execution |
| `bkg-orchestrator` | `TaskGraph` (DAG), `EventBus` (async), `Scheduler` |
| `bkg-tools` | `ledger_summary`, `dump_realm` |
| `bkg-inspector` | realm name registry |
| `bkg-cli` | `bkg` binary — all commands |
| `bkg-testing` | shared test fixtures |

---

## CLI Commands

```bash
# Core commands
bkg init                              # Genesis + Styx ledger
bkg run --input '{"action":"echo","data":"hello"}'
bkg verify                            # Hash-chain verification
bkg replay                            # Reconstruct ledger state
bkg status                            # System state as JSON
bkg isolate                           # Quarantine corrupted branch

# LLM chat (★ new)
bkg chat                              # Interactive session (auto-detects provider)
bkg chat --prompt "..."               # Non-interactive single prompt
bkg chat --model llama3               # Override model
bkg chat --permission read-only       # Restrict tool access
bkg chat --session ./session.jsonl    # Resume saved session

# Agent management (★ new)
bkg agent list
bkg agent spawn --name my-agent --permission workspace-write
bkg agent show <uuid>
```

### Chat slash commands

| Command | Description |
|---|---|
| `/help` | List all slash commands |
| `/status` | BKG system status |
| `/verify` | Hash-chain verification |
| `/replay` | Reconstruct ledger state |
| `/model [name]` | Get or switch LLM model |
| `/system [text]` | Get or set system prompt |
| `/clear` | Clear conversation history |
| `/history` | Print conversation history |
| `/permission [mode]` | Get or set permission mode |
| `/export` | Save session to JSONL |
| `/quit` | Exit |

### Chat provider detection

1. `ANTHROPIC_API_KEY` → Anthropic Claude
2. `OPENAI_API_KEY` → OpenAI-compatible endpoint
3. `OLLAMA_HOST` or default → Ollama at `localhost:11434`

---

## PermissionEnforcer (★ new, `domains/katoptron/verifier/enforcer.rs`)

Single source of truth for all permission logic.

| Mode | bash / write_file | dangerously_allow_any |
|---|---|---|
| `read-only` | ✗ Deny | ✗ Deny |
| `workspace-write` | ✓ Allow | ⚠ Prompt |
| `danger-full-access` | ✓ Allow | ✓ Allow |

---

## Key Invariants

1. Event-first — all mutations derive from Styx events
2. Genesis Lock — genesis hash is immutable
3. Realm Isolation — one entity, one realm
4. Hash chaining — every event carries predecessor hash
5. No hidden state — everything is in the ledger
6. Determinism — same seed + same ledger = same output
7. SWD is the audit protocol
8. Capsules are isolated versioned containers

---

## Verification

```
cargo test --workspace      → all pass
cargo clippy -- -D warnings → clean
```

- [x] `bkg init` → deterministic genesis
- [x] `bkg run` → SWD-audited execution
- [x] `bkg verify` → hash-chain verification
- [x] `bkg replay` → deterministic reconstruction
- [x] `bkg status` → JSON system state
- [x] `bkg chat` → LLM session + slash commands
- [x] `bkg agent spawn` → Telum sandbox agent

---

*BKG v0.1.0 — DELPHOS architecture*
*Single source of truth. One module, one location.*
