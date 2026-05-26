# bkg-snapshot

**World snapshots. Fork. Export. Restore.**

A snapshot is a point-in-time frozen copy of the complete world state.
Fork creates a new divergent timeline. Restore replaces current state.

## Key Types

| Type | Purpose |
|---|---|
| `RealitySnapshot` | Full world state + checksum + fork lineage |
| `RealmSnapshot` | Per-realm frozen state |
| `TimelineSnapshot` | Per-timeline frozen state with event range |
| `SnapshotId` | UUID-based stable identifier |

## Invariant

`RealitySnapshot::verify()` uses the same hash algorithm as `EventLedger`.
If `verify() = false` → snapshot is corrupted → restore from previous.
`fork()` always produces a new `SnapshotId` with `parent_snapshot_id` set.
