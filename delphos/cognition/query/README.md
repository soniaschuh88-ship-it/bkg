# bkg-query

**BQL — BKG Query Language. Deterministic queries over world state.**

```sql
SELECT tasks WHERE status = "blocked" AND dependency.depth > 3 ORDER BY entropy DESC LIMIT 20
```

## Key Types

| Type | Purpose |
|---|---|
| `BqlAst` | Abstract syntax tree |
| `BqlParser` | Hand-written parser (deterministic, no regex) |
| `BqlExecutor` | Executes against ECS world + projection cache |
| `BqlPlanner` | Query plan optimization |

## Invariant

BQL queries are READ-ONLY. They never mutate state.
Results are snapshots — not live references.
