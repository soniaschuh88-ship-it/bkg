# bkg-enforce

**Hard enforcement. No bypass possible. Structurally enforced.**

Sealed traits prevent external implementations. `InvariantGuard` enforces invariants
at call sites. `NoBypass<T>` requires a `PipelinePassport` to unwrap.

## Key Types

| Type | Purpose |
|---|---|
| `Sealed` + `SealedImpl` | External crates cannot implement sealed traits |
| `InvariantGuard` | 7 named invariants — all return `Result`, never panic |
| `NoBypass<T>` | Value requires `PipelinePassport` to unwrap |
| `PipelinePassport` | Proof that an event passed through `EventPipeline` |
| `WorkspaceLints` | `unsafe_code=forbid`, `dead_code=deny`, etc. documented as policy |

## 7 Named Invariants

```
require_event_id()              no mutation without event
require_same_realm()            realm isolation
require_monotone_lamport()      clock monotone
require_monotone_version()      state version monotone
require_entity_id()             non-empty entity IDs
require_matching_checksum()     integrity verification
require_realm_state_exists()    no null realm state
```

## Lint Policy

```toml
unsafe_code = "forbid"    # breaks determinism
dead_code = "deny"        # untestable drift
dbg_macro = "deny"        # side effects in committed code
print_stdout = "deny"     # use structured logging
```
