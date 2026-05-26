# bkg-ecs

**Deterministic sparse-archetype Entity Component System.**

All DELPHOS world entities are ECS entities.
Iteration order is ALWAYS stable (BTreeMap). No HashMap nondeterminism.

## Key Types

| Type | Purpose |
|---|---|
| `World` | Entity registry + component stores |
| `Entity` | `{ id: u64, generation: u32 }` |
| `ComponentStore` | `type_name → BTreeMap<entity_id, Value>` |
| `Archetype` | BTreeSet of component types |
| `SystemRunner` | Sequential system execution (deterministic) |
| `Query` | Filtered entity iteration |

## Invariant

- Iteration order: always stable (BTreeMap key order)
- Generation IDs prevent use-after-free
- No random hashing anywhere in the ECS
