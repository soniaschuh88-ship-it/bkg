# bkg-telemetry

**Model call tracking. Quota monitoring. Cost metrics.**

Every LLM call is logged. Quotas are enforced per provider.
All metrics are append-only events in the telemetry ledger.

## Key Types

| Type | Purpose |
|---|---|
| `ModelCallEvent` | `{ provider, model, tokens, latency_ms, cost_cents }` |
| `QuotaMonitor` | Per-provider usage tracking |
| `TelemetrySummary` | Aggregated stats for display |
