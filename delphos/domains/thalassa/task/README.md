# bkg-task

**Task capsules. The atomic unit of agent work.**

Every task lives at `.bkg/tasks/{id}/`. Its full history is in a per-task event ledger.
State is reconstructed from events via `bkg-state Reducer`. Never stored directly.

## Key Types

| Type | Purpose |
|---|---|
| `Task` | `{ id, title, status, deps, capsule_path }` |
| `TaskStatus` | `planning → todo → in-progress → review → done → archived` |
| `DependencyGraph` | DAG with cycle detection + topological sort |
| `TaskCapsule` | `.bkg/tasks/{id}/` filesystem layout |

## Invariants

- Task state is NEVER stored — always reconstructed from ledger
- DAG must be acyclic — cycle detection on every insert
- T-IDs are deterministic (`DeterministicId::derive`)
