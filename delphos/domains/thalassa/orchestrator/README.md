# bkg-orchestrator

**Task graph. Event bus. Central coordination hub.**

The orchestrator manages the flow of tasks through the system.
It does NOT execute tasks — it coordinates their execution.

## Key Types

| Type | Purpose |
|---|---|
| `TaskGraph` | DAG of tasks with dependency edges |
| `EventBus` | Inter-realm event routing |
| `SchedulerBridge` | Bridge to `bkg-scheduler` |
