# BKG — Causal Data Flow

> Every state change is caused by an event. Every event is proven.

---

## Full Causal Chain

```
Operator action
     │
     ▼
DomainEvent<T>  (typed, signed, causal_parent, schema_id, lamport)
     │
     ▼  EventLedger.append() — only AFTER full pipeline succeeds
EventLedger  (BLAKE3 hash-chain: chain_n = hash(id ∥ hash ∥ prev))
     │  Invariant: ledger.len() == state.version
     │
     ▼  EventPipeline.process()
     │
     ├─ Stage 1: validate_abi()         ABI major version check
     ├─ Stage 2: validate_schema()      EventSchemaRegistry lookup
     ├─ Stage 3: validate_clock()       lamport monotone + duplicate
     ├─ Stage 4: validate_capability()  CapabilityGrant check
     └─ Stage 5: validate_causal()      causal parent processed?
     │
     ▼  KernelDecision: Allow | Reject | Transform
     │
     ▼  StateTransitionFn<E>(state, event) → new_state
     │  Invariants: version+1, same realm_id
     │
     ▼  MaterializerKernel.stamp()
     │  Issues KernelStamp — required for valid projections
     │
     ▼  ProjectionView<T>  (sealed, read-only)
     │  data() → &T  (no data_mut())
     │
     ▼  ProjectionCache.insert()
     │
     ▼  BQL query / UI render
```

---

## Replay Path

```
EventLedger  (source of truth — the only source)
     │
     ▼  ReplaySession.apply(reducer, event_id, schema_id, lamport, event)
     │  for each entry in ledger
     │
     ▼  ReplayIdentityVerifier.verify(original_log, rebuilt_state)
     │
     ├─ Confirmed { event_range, final_checksum }  → system is deterministic
     └─ Diverged  { original_checksum, rebuilt_checksum } → HALT
```

---

## Cross-Realm Communication Path

```
Realm A                          RealmBus                         Realm B
     │                               │                               │
     ├─ bus.send(src, tgt, class,    │                               │
     │   payload_type, payload)      │                               │
     │                               ▼                               │
     │                    LaneRouter (priority queue)                │
     │                    Critical → High → Normal → Background      │
     │                               │                               │
     │                               ▼  bus.recv(target)             │
     │                               │                               ├─ EventPipeline.process()
     │                               │                               ├─ apply reducer
     │                               │                               └─ update state
```

---

## Projection Rebuild Path

When a projection is stale or missing:

```
EventLedger  (source of truth)
     │
     ▼  for each entry in range(from_lamport, to_lamport)
StateTransitionFn<E> applied
     │
     ▼  candidate_state
MaterializerKernel.stamp(projection_id, realm_id, event_range, data)
     │  produces ProjectionContract with KernelStamp
     │
     ▼  ProjectionContract.verify(data)
     │  ChecksumMismatch | NoKernelStamp → reject
     │
     ▼  RebuildProof { identity_confirmed: true/false }
ProjectionCache.insert(view)
```
