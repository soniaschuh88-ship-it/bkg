# bkg-identity

**Deterministic lineage IDs. Timeline fork tracking.**

`DeterministicId::derive(seed, lineage, realm)` always produces the same ID
for the same inputs. This enables reproducible replay.

## Key Types

| Type | Purpose |
|---|---|
| `DeterministicId` | `derive(seed, lineage, realm)` → stable hex ID |
| `AncestryChain` | Ordered list of LineageNodes root → leaf |
| `LineageNode` | `{ id, realm, label, parent_id }` |
| `RealmIdentity` | `genesis(realm, hash, label)` + `fork(label)` |

## Invariant

`DeterministicId::derive(same_inputs)` = same ID, always.
`RealmIdentity::fork()` produces the same child ID for the same parent + label.
