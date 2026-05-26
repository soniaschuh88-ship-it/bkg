# bkg-runtime

**AgentRuntime. The Telum sandbox. Capability-gated execution.**

All agent tool calls go through the runtime.
No direct filesystem or network access without a `CapabilityGrant`.

## Key Types

| Type | Purpose |
|---|---|
| `AgentRuntime` | Capability-gated execution environment |
| `TelumSandbox` | Isolated execution context per agent |
| `RuntimeEvent` | Typed event emitted for every tool call |

## Invariant

- Every tool call produces a `RuntimeEvent` in the ledger
- No tool call is replay-skipped — all are deterministically recorded
