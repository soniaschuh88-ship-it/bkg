# bkg-eval

**Task evaluation scorecards. Evidence collection. Batch evaluation.**

Evaluations are triggered on task completion.
Scores are deterministic given the same evidence.

## Key Types

| Type | Purpose |
|---|---|
| `Scorecard` | Weighted category scores → overall grade |
| `EvalScore` | `f64 ∈ [0,1]` + band `A/B/C/D/F` |
| `EvalEvidence` | Signals + AI commentary |
| `EvalBatch` | Scheduled evaluation over a window of tasks |

## Scoring

```
A: ≥0.90   B: ≥0.75   C: ≥0.60   D: ≥0.40   F: <0.40
```
