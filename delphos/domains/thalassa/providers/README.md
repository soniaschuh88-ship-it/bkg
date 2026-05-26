# bkg-providers

**13 LLM providers. Free detection. Tier metadata. No lock-in.**

All providers expose the same interface. Free-only filter enforced by default.
Provider selection falls back gracefully.

## 13 Providers

| Provider | Free Tier |
|---|---|
| Anthropic | Paid |
| OpenAI | Paid |
| OpenRouter | Mixed |
| SambaNova | Free |
| LLM7 | Free |
| Kilo | Free |
| Cline | Mixed |
| + 6 more | Various |

## Invariant

- Provider calls are NEVER made during replay
- All calls are logged to `bkg-telemetry` for quota tracking
- Free-only filter is opt-out per provider
