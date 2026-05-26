# bkg-world

**The Causal World Model. The true kernel. Everything converges here.**

```
ECS World + Relations + Causality + Temporal flow + Entropy + Intent + Constraints
```

All projections are views of the world. The world is the source of truth.

## Key Types

| Type | Purpose |
|---|---|
| `WorldGraph` | BTreeSet nodes + BTreeMap entity_types + Vec edges |
| `World` | Versioned, queryable, relation-aware |
| `RelationKind` | `DependsOn / BlockedBy / OwnedBy / PartOf / CausedBy` |
| `CausalChain` | `cause_event → effect_entity` links |
| `WorldQuery` | Filtered entity queries (entity_type, relation) |

## Invariant

World version increments on every `add_entity()` and `add_relation()`.
All relations are directional. No undirected edges.
