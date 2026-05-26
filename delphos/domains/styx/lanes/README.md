# bkg-lanes

**Realm Bus IPC fabric. Deterministic inter-realm transport.**

All cross-realm messages route through `RealmBus`.
No direct realm-to-realm calls. Priority-ordered delivery.

## Lane Classes

```
Critical  → capacity=32,  latency=10ms,  no drop on overflow (blocks)
High      → capacity=128, latency=100ms, no drop on overflow (blocks)
Normal    → capacity=256, latency=500ms, no drop on overflow (blocks)
Background → capacity=512, latency=5s,   DROP on overflow
```

## Key Types

| Type | Purpose |
|---|---|
| `RealmBus` | Top-level coordinator |
| `LaneRouter` | Priority-ordered routing |
| `BusPacket` | Signed, sequenced, replayable with `payload_hash` |
| `BackpressureController` | Tracks queue depths |
| `QosPolicy` | Capacity + latency target per lane |

## Delivery Order

`recv(target)` drains: Critical → High → Normal → Background
