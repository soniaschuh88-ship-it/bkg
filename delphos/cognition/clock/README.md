# bkg-clock

**Deterministic time. No `SystemTime::now()` in business logic. Ever.**

`SequencedInstant` uses lamport counters for ordering.
`wall_nanos` is display-only and NEVER used for causal ordering.

## Key Types

| Type | Purpose |
|---|---|
| `SequencedInstant` | `(realm_id, lamport, wall_nanos_display_only)` |
| `VectorClock` | Per-realm causality tracking |
| `CausalTime` | Total causal ordering across realms |
| `RealmClock` | Deterministic tick source |

## Invariant

```rust
// WRONG — breaks replay:
let now = SystemTime::now();

// RIGHT:
let tick = realm_clock.next_tick();  // deterministic lamport
```

Duplicate lamport in same realm = determinism failure → halt.
