# bkg-diff

**Reality diff engine. State/graph/capsule/timeline diffs.**

Diffs are used for: UI change highlights, merge conflict detection,
forensic replay analysis, and mesh sync verification.

## Key Types

| Type | Purpose |
|---|---|
| `StateDiff` | `Added / Removed / Modified` per entity key |
| `GraphDiff` | Node/edge additions and removals |
| `CapsuleDiff` | Version comparison with checksum change detection |
| `CausalTrace` | Who changed what, when, via which event |

## Invariant

All diff operations are PURE functions (no side effects).
The same before/after states always produce the same diff.
