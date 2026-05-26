# bkg-entropy

**System entropy, pressure, heat, stability metrics.**

Telemetry physics: system properties as physical observables.

## Metrics

| Metric | Formula |
|---|---|
| Entropy | `blocked_nodes / total_nodes` |
| Pressure | `active_agents / max_agents` |
| Heat | `error_rate × 10` |
| Stability | `1 - heat - entropy×0.5` |

## Health Labels

```
stability > 0.8  → healthy
stability > 0.5  → degraded
stability ≤ 0.5  → critical
```
