# bkg-projection

**ProjectionCache. Materializer. Index. Subscriptions.**

Projections are disposable read models built from events.
If stale: rebuild from ledger. Never the source of truth.

## Key Types

| Type | Purpose |
|---|---|
| `ProjectionCache` | BTreeMap keyed by `realm/projection_id` |
| `Materializer` | Rebuilds stale projections |
| `ProjectionIndex` | `field_value → BTreeSet<entity_id>` |
| `ProjectionSubscriber` | Live UI push subscriptions |

## Invariant

- Cache miss always returns `is_stale = true` (rebuild-first)
- `build_count` tracked for observability
- Projections must NEVER become the source of truth
