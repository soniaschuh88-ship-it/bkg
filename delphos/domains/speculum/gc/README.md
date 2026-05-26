# bkg-gc

**Causal garbage collection. Prevents "10 TB replay startup".**

Old events are compacted into snapshots. Projections are pruned on invalidation.
Timelines are frozen when no longer active.

## GC Pressure Levels

```
None     → < 10,000 events
Low      → < 100,000 events
Medium   → < 500,000 events  (compact threshold)
High     → < 2,000,000 events (compact threshold)
Critical → ≥ 2,000,000 events (compact threshold)
```

## Key Types

| Type | Purpose |
|---|---|
| `GcPolicy` | Retention + auto_compact + compact_threshold_mb |
| `GcPressure` | 5-level enum |
| `GcRun` | Executes compaction plan |
| `CompactionResult` | Events compacted, snapshots sealed, bytes freed |

## Invariant

GC never removes events that are still referenced by active projections.
`min_events_to_keep` is always respected.
