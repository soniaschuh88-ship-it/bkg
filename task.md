# BKG — task.md

> **System ethic**: *Single source of truth. One module, one location.*
> Every feature lives in exactly one place. No duplication anywhere in the workspace.

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
    ├── threshold/cli/              ← bkg-cli              : bkg binary, all commands
    │
    ├── cognition/
    │   ├── core/                   ← bkg-core             : typed IDs, Hash256, BkgError
    │   ├── kernel/                 ← bkg-kernel           : Genesis, RealmRouter
    │   ├── event/                  ← bkg-event            : Event, EventLedger (mem+file)
    │   ├── contracts/              ← bkg-contracts        : CausalContract (cross-realm)
    │   └── protocol/               ← bkg-acp              : ACP JSON-RPC 2.0 adapter ★
    │
    └── domains/
        ├── thalassa/
        │   ├── runtime/            ← bkg-runtime          : AgentRuntime (Telum sandbox)
        │   ├── orchestrator/       ← bkg-orchestrator     : TaskGraph, EventBus, Scheduler
        │   ├── providers/          ← bkg-providers        : 13 LLM providers, pi-free port ★
        │   ├── agents/             ← bkg-agents           : AgentId, status, credentials ★
        │   ├── session/            ← bkg-session          : SessionManager, UniversalEvent ★
        │   └── exec/               ← bkg-exec             : bash, file, grep, glob tools
        ├── arche/
        │   ├── capsule/            ← bkg-capsule          : immutable versioned containers
        │   └── store/              ← bkg-store            : sled + in-memory persistence
        ├── styx/
        │   ├── provider/           ← bkg-swd              : SwdEngine lifecycle
        │   └── tools/              ← bkg-tools            : ledger inspection
        ├── katoptron/
        │   ├── crypto/             ← bkg-crypto           : BLAKE3, Ed25519
        │   ├── verifier/           ← bkg-verifier         : hash-chain, PermissionEnforcer ★
        │   └── telemetry/          ← bkg-telemetry        : model call tracking, quota ★
        ├── anamnesis/
        │   └── policy/             ← bkg-policy           : PolicyEngine
        └── mnemos/
            ├── memory/             ← bkg-memory           : causal graph
            └── replay/             ← bkg-replay           : ReplayEngine, divergence detection

    └── reflection/
        └── ui/
            └── atlantean/          ← bkg-atlantean        : cyberpunk/Atlantis web UI ★
```

---

## Crates (24 total)

| Crate | Responsibility | Source |
|---|---|---|
| `bkg-core` | typed IDs, Hash256, BkgError | BKG-native |
| `bkg-crypto` | BLAKE3, Ed25519, seed derivation | BKG-native |
| `bkg-event` | Event, hash-chained EventLedger | BKG-native |
| `bkg-contracts` | CausalContract — cross-realm only | BKG-native |
| `bkg-kernel` | Genesis lock, RealmRouter | BKG-native |
| `bkg-swd` | SwdEngine — audit protocol | BKG-native |
| `bkg-capsule` | Capsule + CapsuleManager | BKG-native |
| `bkg-store` | InMemoryStore + SledStore | BKG-native |
| `bkg-memory` | MemoryGraph — impact × recurrence × depth | BKG-native |
| `bkg-replay` | ReplayEngine, DivergenceDetector | BKG-native |
| `bkg-verifier` | hash-chain, PermissionEnforcer | BKG-native + new |
| `bkg-policy` | PolicyEngine + built-in policies | BKG-native |
| `bkg-runtime` | AgentRuntime (Telum sandbox) | BKG-native |
| `bkg-orchestrator` | TaskGraph (DAG), EventBus, Scheduler | BKG-native |
| `bkg-exec` | bash, file, grep, glob tools | BKG-native |
| `bkg-tools` | ledger_summary, dump_realm | BKG-native |
| `bkg-inspector` | realm name registry | BKG-native |
| `bkg-providers` | 13 LLM providers, free detection, toggles | **pi-free port** |
| `bkg-telemetry` | model call tracking, quota monitor | **pi-free port** |
| `bkg-agents` | AgentId (7 agents), credentials, status | **sandbox-agent port** |
| `bkg-session` | SessionManager, UniversalEvent, SSE | **sandbox-agent port** |
| `bkg-acp` | ACP JSON-RPC 2.0, AgentBridge, InferenceProxy | **sandbox-agent port** |
| `bkg-atlantean` | cyberpunk/Atlantis web dashboard | **new** |
| `bkg-cli` | `bkg` binary — all commands | **extended** |

---

## pi-free Integration (bkg-providers + bkg-telemetry)

### 13 Providers

| Provider | Tier | Auth |
|---|---|---|
| Ollama | private/free | none (local) |
| NVIDIA NIM | freemium | `NVIDIA_API_KEY` |
| OpenRouter | freemium | `OPENROUTER_API_KEY` |
| SambaNova | free | `SAMBANOVA_API_KEY` |
| LLM7 | free | none |
| Kilo | free/OAuth | `KILO_API_KEY` |
| Cline | free/OAuth | `CLINE_API_KEY` |
| ZenMux | paid | `ZENMUX_API_KEY` |
| CrofAI | paid/free-named | `CROFAI_API_KEY` |
| Codestral | free experiment | `CODESTRAL_API_KEY` |
| DeepInfra | freemium | `DEEPINFRA_API_KEY` |
| Together AI | freemium | `TOGETHER_API_KEY` |
| Novita AI | freemium | `NOVITA_API_KEY` |

### Fallback Chain (same for providers + agents)
1. User's own key (BKG user config)
2. Admin global key (`~/.bkg/global-providers.json`)
3. Environment variable
4. Anonymous (Kilo + LLM7 — no key needed)

---

## sandbox-agent Integration (bkg-agents + bkg-session + bkg-acp)

### 7 Supported Agents

| BKG ID | Upstream | Modes |
|---|---|---|
| `claude` | Anthropic Claude Code | Default, Bypass, **BkgSupervised** |
| `codex` | OpenAI Codex | Default, PlanMode, **BkgSupervised** |
| `opencode` | OpenCode | Default, **BkgSupervised** |
| `amp` | Amp | Default, Bypass |
| `pi` | Pi | Default, **BkgSupervised** |
| `cursor` | Cursor | Default |
| `mock` | BKG Mock | All modes |

**BkgSupervised** is a BKG-native mode that enforces Plan→Review→Execute workflow gates via `bkg-workflow`.

### UniversalEvent Schema
Every agent's native events are normalized to `UniversalEvent`:
- `started` | `message` | `delta` | `question_asked` | `permission_asked`
- `question_answered` | `permission_decided` | `finished` | `error` | `unknown`

All events have: `id` (offset), `timestamp`, `session_id`, `agent`, `data`.
Replay from any offset is deterministic (BKG invariant: every event is reconstructable).

### ACP Method Registry (`_bkg/` namespace)
24 methods: `session/*`, `agent/*`, `process/*`, `file/*`, `_bkg/*`

---

## bkg-atlantean Dashboard

### Design: Cyberpunk / Atlantis
- Deep void palette (#060a14) + teal (#00d2b4) + gold (#ffd76e) + violet (#8b5cf6)
- Orbitron + Exo 2 fonts, animated particle grid, glowing neon borders
- Glassmorphism panels, holographic gradients
- Responsive (collapses sidebar on mobile)

### Private / Cloud Mode Switch
- **Private**: WebLLM (browser WebGPU, CDN-loaded), Ollama tunnel (`/tunnel/ollama/*`)
- **Cloud**: 13 free providers, fallback chain, free-only toggle

### Pages
| Page | Description |
|---|---|
| Chat | Interactive LLM with /slash commands |
| Providers | All 13 providers, tier, toggle, signup links |
| My Keys | Per-user API keys grouped by tier |
| **Agents** | 7 agents, status, credentials, mode badges ★ |
| **Inspector** | Session browser + live event viewer (SSE) ★ |
| Dashboard | Stats, provider status table, telemetry |
| Admin | Global provider keys, default model, free-only |

### Inspector (ported from sandbox-agent Inspector UI)
- Session list (left panel): all active sessions
- Create session: pick agent + mode (default/bypass/plan_mode/bkg_supervised)
- Event viewer (right panel): live SSE stream + offset-based replay
  - Color-coded by event type
  - Event offset + timestamp
  - Content preview (truncated at 400 chars)
- Send message input at the bottom

### API Endpoints

```
# Mode
GET/PUT  /api/mode
GET      /api/models?mode=
GET      /api/stats
GET      /api/telemetry

# Providers (pi-free)
GET      /providers/list
GET      /providers/:id/models
POST     /providers/proxy              ← fallback chain inference

# Users + Admin
GET/PUT  /user/providers
GET      /user/profile
POST     /user/onboarded
GET/PUT  /admin/globals
POST     /admin/globals/providers
POST     /api-keys/self-register       ← rate-limited (3/hr)

# Agents (sandbox-agent)
GET      /agents/list
GET      /agents/:id/status
POST     /agents/:id/credentials

# Sessions / Inspector (sandbox-agent)
GET/POST /sessions
GET      /sessions/:id
DELETE   /sessions/:id
POST     /sessions/:id/send
GET      /sessions/:id/stream          ← SSE with offset replay

# Tunnel
POST/GET /tunnel/ollama/*              ← reverse-proxy to localhost:11434
```

---

## CLI Commands

```bash
# Core
bkg init   bkg run   bkg verify   bkg replay   bkg status   bkg isolate

# Chat (bkg-atlantean)
bkg chat                      # Interactive LLM (Anthropic/OpenAI/Ollama auto-detect)
bkg chat --prompt "..."       # Non-interactive

# Agent management
bkg agent list
bkg agent spawn --name X --permission workspace-write

# Providers (pi-free)
bkg providers list
bkg providers models <id> [--all]
bkg providers toggle <id>
bkg providers refresh <id|all>
bkg providers telemetry
bkg providers quota
```

### Chat /slash commands

`/help` `/status` `/verify` `/replay` `/model` `/system` `/clear` `/history`
`/permission [mode]` `/export` `/quit`

---

## PermissionEnforcer (bkg-verifier)

| Mode | bash/write | dangerously_allow_any |
|---|---|---|
| `read-only` | ✗ Deny | ✗ Deny |
| `workspace-write` | ✓ Allow | ⚠ Prompt |
| `danger-full-access` | ✓ Allow | ✓ Allow |

---

## Key Invariants

1. Event-first — all mutations derive from Styx events
2. Genesis Lock — immutable; deviations create new timelines
3. Realm Isolation — one entity, one realm
4. Hash chaining — every event carries predecessor hash
5. No hidden state — everything is in the ledger
6. Determinism — same seed + same history = identical output
7. SWD is the primary audit protocol
8. Capsules are isolated versioned containers
9. Single source of truth — one module, one location

---

## Test Coverage

```
cargo test --workspace   → all pass
cargo clippy -- -D warnings  → clean across all 24 crates
```

---

*BKG v0.1.0 — DELPHOS architecture*
*Single source of truth. One module, one location.*
*Integrates: pi-free (LLM providers) + sandbox-agent (agent runtime)*
