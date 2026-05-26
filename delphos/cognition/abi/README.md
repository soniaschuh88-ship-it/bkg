# bkg-abi

**Universal Realm ABI. Every cross-system message is wrapped here.**

`AbiEnvelope<T>` enables version negotiation for mesh + plugin compatibility.
Old mesh nodes, old plugins, old snapshots remain readable.

## Key Types

| Type | Purpose |
|---|---|
| `AbiEnvelope<T>` | Wraps any payload with `abi_version + payload_type + payload_hash` |
| `AbiVersion` | Major.Minor semantic versioning |
| `EventAbiPayload` | Typed event serialization contract |
| `PacketAbiPayload` | IPC packet format for bkg-lanes |
| `CapsuleAbiPayload` | Capsule serialization contract |
| `ProjectionAbiPayload` | Read-model wire format |
| `PluginManifestPayload` | Plugin contribution format |
| `LlmRequestPayload` | LLM request/response normalization |
| `MeshSyncPayload` | Cross-node replication format |

## Version Negotiation

```rust
let env = AbiEnvelope::wrap(payload, "bkg.event.v1")?;
assert!(env.verify_hash()); // tamper-evident
```
