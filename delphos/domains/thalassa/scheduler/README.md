# bkg-scheduler

**Deterministic DAG scheduler. Concurrent agent slot limiting.**

Tasks are scheduled by DAG topological order, then by priority.
Two tasks with shared file scope cannot run concurrently (OverlapGate).

## Key Types

| Type | Purpose |
|---|---|
| `TaskScheduler` | Priority queue + slot semaphore |
| `Priority` | `Urgent > High > Normal > Low` (FIFO tie-break) |
| `OverlapGate` | Prevents tasks with shared file scope running concurrently |
| `TaskLease` | Distributed lease with epoch fencing |
| `AgentSemaphore` | Concurrent agent slot limiting |

## Invariant

- The scheduler is DETERMINISTIC: same DAG + same priorities = same execution order
- OverlapGate prevents file scope collisions
