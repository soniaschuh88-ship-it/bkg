# bkg-replay

**ReplayEngine. Deterministic replay with divergence detection.**

Replays the event ledger from any starting point.
Divergence between original and replayed state → immediate halt.

## Key Types

| Type | Purpose |
|---|---|
| `ReplayEngine` | Drives replay from ledger |
| `ReplayResult` | `Identical / Diverged { at_event_id }` |

## Invariant

`fold(f, S0, [e1..en]) = Sn` — the replay identity must hold.
If `result.is_diverged()` → determinism failure → system halt.
