# bkg-workflow

**Deterministic Plan→Review→Execute workflow engine.**

All task execution goes through a gate system.
Gates produce verdicts; verdicts drive state transitions.

## Key Types

| Type | Purpose |
|---|---|
| `WorkflowGate` | Phase + verdict config + retry limits |
| `Verdict` | `APPROVE \| REVISE \| RETHINK \| UNAVAILABLE` |
| `WorkflowPhase` | `Plan \| PlanReview \| Execute \| ExecuteReview` |
| `ExecutionGraph` | DAG with loops, retries, parallel waves, conditions |
| `WaveExecution` | Parallel sessions with no shared file scope |

## Execution Model

```
Plan → PlanReview (gate) → Execute → ExecuteReview (gate) → Done
                 ↓ RETHINK                       ↓ REVISE
              retry loop                       retry loop
```

UNAVAILABLE → fallback model + stricter instructions
