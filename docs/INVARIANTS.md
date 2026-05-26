# BKG — All Invariants

> Structural invariants are enforced by the type system.
> Runtime invariants return `Result`, never panic in production.

---

## Structural Invariants (compile-time)

### Projection isolation
- `RealmState` is `pub(crate)` — inaccessible outside `bkg-state`
- `ProjectionView<T>` has no `data_mut()` method — mutation is impossible
- `ProjectionSeal` is `pub(crate)` — only `ProjectionFactory` can create views
- `ProjectionFactory` can only be called from within `bkg-state`

### Sealed traits
- `Reducer<E>: Sealed` — external crates cannot implement `Reducer`
- Only workspace crates can satisfy the `Sealed` bound

### Event pipeline
- All events must pass through `EventPipeline.process()` before `Realm::submit_event()`
- `NoBypass<T>` requires a `PipelinePassport` to unwrap (from pipeline only)

### ReplayIdentityProof
- `ReplayIdentityProof` is a structural value — `Diverged` = halt
- Not a test assertion — a first-class type in the state machine

---

## Runtime Invariants (InvariantGuard)

All return `Result<(), InvariantViolated>` — never panic in production.

| Invariant | Check |
|---|---|
| `no-mutation-without-event` | `event_id` must be non-empty |
| `realm-isolation` | `from.realm_id == to.realm_id` in transitions |
| `monotone-lamport` | `next_lamport > prev_lamport` |
| `monotone-version` | `to_version == from_version + 1` |
| `non-empty-entity-id` | entity IDs must not be empty strings |
| `checksum-integrity` | `expected_checksum == actual_checksum` |
| `no-null-realm-state` | realm state must exist before operations |

---

## Kernel State Machine Invariants

| Property | Enforcement |
|---|---|
| TOTAL δ | `kernel_delta(q, σ)` defined for all 522 cells; undefined → Faulted |
| DETERMINISTIC | same (q, σ) → same q' — verified by test over all 522 cells |
| Sealed absorbing | `Sealed + any_input → Sealed` |
| Faulted absorbing | `Faulted + non_recovery → Faulted` |
| Pipeline acyclic | no backward arcs in processing phases (BFS proves it) |
| No rule conflicts | no two rules disagree for same cell |
| Table consistent | algebra agrees with kernel on all 522 cells |

---

## Ledger Invariants

```
ledger.len() == transition_log.len() == state.version
```

- Entries are never modified or deleted (append-only)
- Hash chain: `chain_n = hash(id_n ∥ payload_hash_n ∥ chain_{n-1})`
- Lamport counter is strictly increasing
- Duplicate `event_id` rejected

---

## Synthesis Invariants (SynthesisCycleGuard)

| Invariant | Type | Threshold |
|---|---|---|
| `kernel-alignment` | HARD | 100% agreement with `kernel_delta` |
| `pipeline-acyclic` | HARD | no backward arcs |
| `no-conflicts` | HARD | zero conflicting rule pairs |
| `minimum-coverage` | SOFT | ≥50% of TRANSITION_TABLE |
| `expression-floor` | SOFT | above `EntropyFloor::development()` |

Pinned rules that can never be removed:
`sealed-absorbs-all`, `faulted-absorbs-non-recovery`, `universal-fault`,
`pipeline-advance`, `validation-rejection`

---

## Expressiveness Conservation Law

```
free_fraction(R_t+1) >= free_fraction(R_t) × (1 - 0.10)
free_fraction >= 0.50  (production floor)
free_cells >= 100      (absolute floor)
```

Current canonical spec: **80.1% free** (418/522 cells)

---

## Workspace Code Invariants

```toml
unsafe_code = "forbid"   # no unsafe anywhere
dead_code = "deny"       # zero stubs
dbg_macro = "deny"       # no debug prints in committed code
print_stdout = "deny"    # use structured logging
```
