# BKG Project Organisation Rules
> Single source of truth. One module, one location.

## Directory Map

| Path | Crate | Role |
|------|-------|------|
| `delphos/cognition/core` | bkg-core | Primitive types, IDs, errors |
| `delphos/cognition/kernel` | bkg-kernel | Genesis, RealmRouter, ContractValidator |
| `delphos/cognition/event` | bkg-event | Event type, EventLedger |
| `delphos/cognition/contracts` | bkg-contracts | CausalContract types |
| `delphos/domains/katoptron/crypto` | bkg-crypto | BLAKE3, Ed25519 |
| `delphos/domains/katoptron/verifier` | bkg-verifier | Hash-chain, capsule, drift checks |
| `delphos/domains/styx/provider` | bkg-swd | SWD engine |
| `delphos/domains/styx/tools` | bkg-tools | Ledger inspection |
| `delphos/domains/arche/capsule` | bkg-capsule | Capsule versioning |
| `delphos/domains/arche/store` | bkg-store | State persistence |
| `delphos/domains/thalassa/runtime` | bkg-runtime | Agent execution sandbox |
| `delphos/domains/thalassa/orchestrator` | bkg-orchestrator | Task graph, event bus |
| `delphos/domains/anamnesis/policy` | bkg-policy | Policy enforcement |
| `delphos/domains/mnemos/memory` | bkg-memory | Causal memory graph |
| `delphos/domains/mnemos/replay` | bkg-replay | Deterministic replay |
| `delphos/threshold/cli` | bkg-cli | CLI binary |

## Invariants
- No file lives in two places
- `cargo check --workspace` must pass
- Cross-realm access only via CausalContract + RealmRouter
- All state mutations produce events
- Determinism: same seed + same ledger = same output
