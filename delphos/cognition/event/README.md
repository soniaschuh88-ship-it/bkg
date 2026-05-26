# bkg-event

**Typed events. The only data that flows between realms.**

`TypedEvent<P: EventPayload>` replaces loose `serde_json::Value`.
Every event has a statically-known payload type and a stable schema ID.

## Key Types

| Type | Purpose |
|---|---|
| `TypedEvent<P>` | Typed, replay-safe event |
| `EventPayload` | Trait: `SCHEMA_ID`, `producer_realm()` |
| `DomainEvent` | Legacy untyped event (being migrated) |
| `all_typed_events()` | Registry of all 9 canonical event types |

## 9 Canonical Event Types

```
task.created        TaskCreated { task_id, title, priority, agent_id }
task.status_changed TaskStatusChanged { task_id, from, to, changed_by }
session.started     SessionStarted { session_id, agent_id, mode }
approval.granted    ApprovalGranted { approval_id, kind, granted_by }
approval.rejected   ApprovalRejected { approval_id, reason, rejected_by }
workflow.approved   WorkflowApproved { task_id, phase, feedback }
workflow.failed     WorkflowFailed { task_id, phase, reason }
capability.granted  CapabilityGranted { grantee, capability, granted_by }
snapshot.created    SnapshotCreated { snapshot_id, realm_id, version }
```

## Invariant

- `P: EventPayload` provides `const SCHEMA_ID` — no runtime schema lookup
- `payload_hash` is always computed and verified
- `causal_parent` enforces ordering
