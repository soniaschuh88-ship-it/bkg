# BKG — Getting Started

---

## Prerequisites

- Rust 1.95+ (`rustup update stable`)
- 2GB disk space

---

## Quick Start

```bash
# Clone
git clone https://github.com/soniaschuh88-ship-it/bkg.git
cd bkg

# Build everything (60+ crates)
cargo build --workspace

# Run all tests
cargo test --workspace

# Quality check
cargo clippy --workspace -- -D warnings

# Start the Atlantean dashboard
cargo run -p bkg-atlantean
# → http://localhost:7878
```

---

## Project Initialization

```bash
# Initialize a new BKG project in the current directory
cargo run -p bkg-cli -- init

# This creates:
# .bkg/
#   bkg-central.db       project registry (sled)
#   tasks/               task capsules
#   snapshots/           world snapshots
#   ledger/              event ledger
```

---

## First Event

```rust
use bkg_kernel::{EventPipeline, PipelineConfig, PipelineEvent};
use bkg_core::RealmId;

// Create pipeline
let mut pipeline = EventPipeline::new(PipelineConfig::default());

// Submit an event
let event = PipelineEvent::new(
    "evt-001", "task.created",
    RealmId::Telum, RealmId::Telum,
    1,  // lamport
    serde_json::json!({"task_id": "T-1", "title": "My first task"}),
);

let result = pipeline.process(&event);
assert!(result.decision.is_allow());
```

---

## Using the Realm

```rust
use bkg_kernel::Realm;
use bkg_core::RealmId;

// Create a realm (verifies rule engine at construction)
let mut realm = Realm::open(RealmId::Telum, "my-realm");

// Submit an event atomically
let result = realm.submit_event(&pipeline_event, my_reducer, event_data);
assert!(result.outcome.is_committed());

// Consistency is guaranteed
let proof = realm.verify_consistency();
assert!(proof.consistent);
```

---

## Reading from Projections

```rust
use bkg_state::{ProjectionFactory, KanbanProjection};

// Create a projection view (sealed read-only)
let view = ProjectionFactory::create(
    "kanban", "telum", state_version, checksum,
    KanbanProjection { todo: vec!["T-1".into()], total_tasks: 1, ..Default::default() }
);

// Read-only access only
let data = view.data();
println!("Tasks: {}", data.total_tasks);

// Staleness detection
if view.is_stale(current_checksum) {
    // rebuild from ledger
}
```

---

## Verifying Replay Identity

```rust
use bkg_kernel::{ReplaySession, ReplayIdentityVerifier};

let s0 = RealmState::empty(RealmId::Telum);

// Build original state
let mut original = ReplaySession::from(s0.clone());
original.apply(my_reducer, "e1", "task.created", 1, event_data)?;

// Replay
let mut rebuilt = ReplaySession::from(s0);
rebuilt.apply(my_reducer, "e1", "task.created", 1, event_data)?;

// Verify
let proof = rebuilt.verify_identity(&original.log);
assert!(proof.is_confirmed(), "replay identity must hold");
```

---

## Key Invariants to Remember

1. **Never call `SystemTime::now()`** in business logic — use `bkg-clock`
2. **Never mutate `RealmState` directly** — only via `StateTransitionFn<E>`
3. **Never read `RealmState` in UI** — only via `ProjectionView<T>`
4. **All events through `EventPipeline`** — no bypass
5. **Replay identity is structural** — `ReplayIdentityProof::Diverged` = halt

---

## Documentation

| File | Contents |
|---|---|
| `README.md` | Overview + architecture |
| `docs/ARCHITECTURE.md` | Realm model + data flows |
| `docs/KERNEL_FORMAL_SYSTEM.md` | L0-L12 formal specification |
| `docs/INVARIANTS.md` | All invariants in one place |
| `docs/CAUSAL_FLOW.md` | Complete causal chain diagrams |
| `docs/FEATURES.md` | Complete crate catalog |
| `docs/TASKS.md` | Roadmap + completed work |
| `docs/PROGRESS.md` | Development history + ADRs |
| `docs/rule.md` | Organization rules + directory map |
| `delphos/*/README.md` | Per-crate documentation |
