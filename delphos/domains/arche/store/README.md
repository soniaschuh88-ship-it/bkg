# bkg-store

**Persistent storage backend. sled + in-memory fallback.**

The store is the ONLY place where data outlives a process restart.
All reads/writes go through typed interfaces — no raw SQL.

## Key Types

| Type | Purpose |
|---|---|
| `SledStore` | Persistent sled-backed store |
| `MemoryStore` | In-memory store for testing |
| `StoreKey` | Typed key with realm + entity_type + id |

## Invariant

- All writes are atomic (sled transactions)
- Reads return `Option<T>` — never panic on missing key
