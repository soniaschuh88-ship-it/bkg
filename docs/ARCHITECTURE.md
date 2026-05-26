# BKG Architecture

> Single source of truth. One module, one location.

---

## Overview

BKG is a **deterministic causal operating substrate** with replayable world simulation.

```
Not: agent framework
Not: event-driven microservices
But: deterministic ontology engine where AI agents are inhabitants, not the core
```

---

## Realm Model

DELPHOS is organized into **realms**. Each realm has:
- A `RealmId` (strongly typed enum)
- A `RealmState` (immutable, versioned, checksummed)
- An `EventLedger` (append-only, hash-chained)
- A `KernelMachine` (formal state machine M=(Q,Σ,Λ,δ,λ,q₀))

| Realm | ID | Purpose |
|---|---|---|
| Telum | `RealmId::Telum` | Task execution + agent work |
| Katoptron | `RealmId::Katoptron` | Observation + projection |
| Styx | `RealmId::Styx` | Event provider + IPC |
| Causa | `RealmId::Causa` | Causal governance |
| Arche | `RealmId::Arche` | Persistence + storage |
| Speculum | `RealmId::Speculum` | Verification + audit |
| Anamnesis | `RealmId::Anamnesis` | Policy + memory |
| Mnemos | `RealmId::Mnemos` | Replay |
| Thalassa | `RealmId::Thalassa` | General execution |

---

## Causal Data Flow

```
DomainEvent<T>
    │  typed, signed, causal_parent, schema_id
    ▼
EventLedger
    │  append-only, BLAKE3 hash-chain, tamper-evident
    │  ledger.len() == state.version (invariant)
    ▼
EventPipeline.process()
    │  5 validation stages: ABI → Schema → Clock → Capability → Causal
    │  → KernelDecision: Allow | Reject | Transform
    ▼
StateTransitionFn<E>
    │  fn(&RealmState, E) -> Result<RealmState, TransitionError>
    │  the ONLY state mutation path
    ▼
MaterializerKernel.stamp()
    │  KernelStamp issued — required for valid projections
    ▼
ProjectionView<T>
    │  sealed, read-only, checksum-verified
    │  data() → &T (no data_mut())
    ▼
BQL queries → Atlantean UI
```

---

## The Formal Kernel

The kernel implements a formally specified Mealy machine M=(Q,Σ,Λ,δ,λ,q₀).

**18 phases (Q)**: Genesis → Bootstrapping → Idle → [10 processing phases] → [replay] → Sealed/Faulted

**29 inputs (Σ)**: lifecycle, processing, replay, control signals

**δ**: TOTAL transition function — every (phase, input) has exactly one output  
**λ**: Pure effect function — `KernelEffectIsolated` is `Copy`, no runtime strings

The full 12-layer specification: [`docs/KERNEL_FORMAL_SYSTEM.md`](KERNEL_FORMAL_SYSTEM.md)

---

## Cross-Realm Communication

Realms communicate ONLY via:
1. `RealmBus` (bkg-lanes) — priority-ordered message bus
2. `CausalContract` (bkg-contracts) — typed cross-realm event permissions

Direct realm-to-realm calls are forbidden.

---

## Projection Isolation

```
RealmState is pub(crate) — inaccessible outside bkg-state
ProjectionView<T> is the ONLY public read interface
ProjectionFactory is pub(crate) — only Materializer can construct views
data_mut() does not exist — mutation is structurally impossible
```

---

## Atomic Commit Protocol

```
Realm::submit_event() is atomic:
  Stage 1: EventPipeline.process()     — no mutation
  Stage 2: reducer(state, event)       — no mutation, produce candidate
  Stage 3: MaterializerKernel.stamp()  — no mutation, produce contract
  Commit:  KernelMachine.step() ×N     — advance phase
           self.state = candidate      — apply state
           ledger.append()             — LAST (commit point)

Failure at any stage → zero observable change.
```

---

## Testing Strategy

| Layer | Test Type | Tool |
|---|---|---|
| Unit | Per-module behavior | `#[test]` |
| Integration | Cross-crate pipeline | `integration.rs` |
| Proof | Compile-time structural | type system |
| Formal | All 522 δ cells | exhaustive enumeration |

`DriftDetector::canonical_algebra_agrees_with_kernel_everywhere` runs on every `cargo test`.
