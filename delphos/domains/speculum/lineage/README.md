# bkg-lineage

**Timeline ancestry graph. Fork tracking. Common ancestor.**

When timelines diverge (forks), their ancestry is tracked here.
Common ancestor lookup enables diff and merge operations.

## Key Types

| Type | Purpose |
|---|---|
| `LineageGraph` | BTreeSet nodes + Vec edges |
| `ForkRecord` | `{ parent, child, reason, label }` |
| `ForkReason` | `Experiment / Recovery / Branching / Rollback` |

## Usage

```rust
graph.record_fork(&fork);
graph.ancestors_of("snap-2");   // ["snap-1"]
graph.descendants_of("snap-1"); // ["snap-2"]
```
