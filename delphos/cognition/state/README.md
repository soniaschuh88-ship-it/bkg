# bkg-state

**Immutable realm state. The only source of truth for data at rest.**

`RealmState` is the single canonical state type. It is NEVER mutated directly.
All mutations go through `StateTransitionFn<E>`. Projections are sealed read-only views.

## Key Types

| Type | Purpose |
|---|---|
| `RealmState` | Immutable, version-monotone, checksummed |
| `StateTransitionFn<E>` | `fn(&RealmState, E) -> Result<RealmState, TransitionError>` |
| `ProjectionView<T>` | Sealed read-only projection — cannot be mutated |
| `ProjectionFactory` | The ONLY way to create `ProjectionView<T>` instances |
| `MaterializerKernel` | Issues `KernelStamp` — required for valid projections |
| `EventRange` | `{ from_lamport, to_lamport, event_count }` |
| `ProjectionChecksum` | Deterministic hash of projection data |
| `ProjectionContract` | Formal contract: checksum + event_range + kernel_stamp |
| `RebuildProof` | Proof that a projection was rebuilt from ledger alone |
| `InvariantGuard` | 7 named runtime invariants (return Result, never panic) |

## Projection Isolation (structural)

```rust
// ProjectionView<T> — the only public read interface
// data_mut() does not exist — mutation is structurally impossible
let view: ProjectionView<KanbanProjection> = factory.create(...);
view.data().total_tasks   // ✅ read-only
// view.data_mut()        // ❌ does not compile
```

## Rebuild Guarantee

Every projection must be rebuildable from the ledger alone.
`MaterializerKernel::stamp()` issues a `KernelStamp` — projections without it are rejected.
