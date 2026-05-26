# bkg-operator

**Operator consciousness. Intent tracking. Adaptive orchestration.**

Models the human operator's current focus, intent, and interaction history.
Used to adapt agent behavior and prioritize tasks.

## Key Types

| Type | Purpose |
|---|---|
| `OperatorIntent` | Inferred intent with confidence score |
| `IntentKind` | `CreateTask / ReviewCode / DebugSystem / PlanMission / ...` |
| `AttentionMap` | `entity_id → attention_score` (with decay) |
| `InteractionHistory` | Last 200 interactions (circular buffer) |

## Intent Inference

```
confidence >= 0.7 → high-confidence intent
```

Attention decays by factor per cycle: `score *= damping_factor`
