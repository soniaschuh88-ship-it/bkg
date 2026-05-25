# BKG — Deterministic Multi-Realm Execution System

**v0.1.0** · Rust workspace · DELPHOS architecture

BKG is a deterministic, replayable multi-realm execution system where:
- every state emerges from events
- every event is fully reconstructable  
- every deviation creates a new timeline
- **SWD** is the primary audit protocol
- **Capsules** are isolated, versioned state containers

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# Try the CLI
cargo run --bin bkg -- init
cargo run --bin bkg -- status
cargo run --bin bkg -- run --input '{"action":"echo","data":"hello BKG"}'
cargo run --bin bkg -- verify
cargo run --bin bkg -- replay
```

## Realms

| Realm | Function |
|-------|----------|
| Telum | Execution — Agents, Runtime |
| Causa | State — Capsules, Snapshots |
| Styx | Event Ledger — append-only DAG |
| Speculum | Verification — Trust, Integrity |
| Mensa | Memory Graph — semantic memory |
| Katoptron | Observation — UI, Debugging |
| Anamnesis | Policy — compliance rules |

See [docs/rule.md](docs/rule.md) for the full structure.
