# Contributing to BKG

> Single source of truth. One module, one location.

---

## Core Rules

1. **One concept, one crate** — if it needs to live in two places, the design is wrong
2. **No direct state mutation** — only via `StateTransitionFn<E>::apply()`
3. **No `SystemTime::now()`** — use `bkg-clock::SequencedInstant`
4. **Replay-safe** — same events + same reducers = same final state, always
5. **Projection-only UI** — UI reads `ProjectionView<T>`, never `RealmState`
6. **All events through pipeline** — `EventPipeline.process()` before `Realm::submit_event()`

---

## Before You Start

```bash
# Verify the workspace is clean
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# These must all pass. If they don't, fix before proceeding.
```

---

## Adding a New Crate

1. Create directory under the correct realm: `delphos/domains/<realm>/<name>/`
2. Add `Cargo.toml` with workspace inheritance:
   ```toml
   [package]
   name = "bkg-<name>"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   
   [dependencies]
   bkg-core.workspace = true
   serde.workspace = true
   ```
3. Add to workspace `Cargo.toml` members list
4. Implement with zero `allow(dead_code)` — real code only
5. Add tests: minimum coverage for all public types
6. Add `README.md` in the crate directory

---

## Adding a New Event Type

1. Add struct implementing `EventPayload` in `bkg-event/src/typed_event.rs`
2. Add to `all_typed_events()` registry
3. Implement the corresponding reducer in the domain crate
4. Add to `EventSchemaRegistry`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyNewEvent { pub field: String }

impl EventPayload for MyNewEvent {
    const SCHEMA_ID: &'static str = "my.new.event";
    fn producer_realm() -> RealmId { RealmId::Telum }
}
```

---

## Testing Requirements

- Every public type must have at least one test
- Every error path must have at least one test
- Tests must be deterministic (no random, no SystemTime)
- Use `#[cfg(test)]` — never conditional compilation in production code

```bash
# Run tests for your crate
cargo test -p bkg-<name>

# Run the full suite
cargo test --workspace
```

---

## Clippy Requirements

```bash
cargo clippy -p bkg-<name> --no-deps -- -D warnings
```

All clippy warnings are errors. Common violations to avoid:
- Unused imports (fix immediately)
- `&'static str` in `Serialize` structs (use `String`)
- Cross-product predicates (use union-of-pairs)
- `default()` method that shadows `Default::default` (rename to `standard()`)

---

## Commit Convention

```
feat(<crate>): description — N tests

<body: what was built, why, empirical results>

Co-Authored-By: Sonia Schuh <soniaschuh88@gmail.com>
```

---

## Pull Request Checklist

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Zero `allow(dead_code)` in new code
- [ ] `README.md` updated for changed crates
- [ ] `docs/` updated if architecture changes
