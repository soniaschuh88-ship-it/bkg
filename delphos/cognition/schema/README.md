# bkg-schema

**Event schema registry. Enables replay-safe migrations.**

Without this: replay migrations, cross-version mesh, ABI upgrades break.
Every event type is registered with its schema ID, version, and migration strategy.

## Key Types

| Type | Purpose |
|---|---|
| `EventSchemaRegistry` | Global catalog of all event schemas |
| `EventSchema` | `{ id, version, producer_realm, migration_strategy }` |
| `SchemaVersion` | Major.Minor semantic versioning |
| `MigrationStrategy` | Passthrough \| Skip \| Reject \| Transform |

## Migration Strategies

```
Passthrough — same payload, no transformation
Skip        — discard old event (no replay contribution)
Reject      — block replay if old version encountered
Transform   — transformer_id maps old → new payload
```
