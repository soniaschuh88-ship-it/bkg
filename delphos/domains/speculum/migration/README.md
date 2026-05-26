# bkg-migration

**Replay-safe schema migrations. Version tracking per crate.**

When event schemas change, old events in the ledger must still be readable.
`MigrationRunner` applies transformations safely.

## Key Types

| Type | Purpose |
|---|---|
| `VersionMap` | `crate_name → current_schema_version` |
| `MigrationStep` | `{ crate, from_version, to_version, description }` |
| `MigrationPlan` | Ordered set of steps |
| `MigrationRunner` | Executes plan: Apply / Skip / Fail |

## Migration Outcomes

```
Applied   — migration applied, version bumped
Skipped   — already at target version
Failed    — wrong from_version (schema inconsistency)
```
