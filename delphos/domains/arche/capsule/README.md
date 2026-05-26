# bkg-capsule

**The fundamental unit of persistent data. Versioned. Lifecycle-managed.**

Every capsule has a deterministic ID derived from its lineage.
The lifecycle state machine ensures correct transitions.

## Lifecycle States

```
Created → Mounted → Active → Frozen → Forked → Archived → Corrupted → Recovered
```

## Key Types

| Type | Purpose |
|---|---|
| `Capsule` | `{ id, version, realm_id, entity_type, payload_hash }` |
| `CapsuleLifecycle` | State machine for capsule transitions |
| `CapsuleId` | Deterministic ID from lineage |
