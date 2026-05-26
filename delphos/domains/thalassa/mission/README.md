# bkg-mission

**Mission → Milestone → Slice → Task hierarchy.**

Missions are the highest-level planning unit. An autopilot activates
the next slice automatically on slice completion.

## Hierarchy

```
Mission
  └── Milestone (ordered)
        └── Slice (parallel feature set)
              └── Feature (acceptance criteria + fix budget)
                    └── Task
```

## Key Types

| Type | Purpose |
|---|---|
| `Mission` | Top-level goal |
| `Milestone` | Ordered set of slices |
| `Slice` | Parallel features with shared scope partition |
| `MissionAutopilot` | Auto-activate next slice on completion |
| `FixBudget` | Max retries for failed features |
