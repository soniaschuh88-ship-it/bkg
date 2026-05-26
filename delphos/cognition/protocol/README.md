# bkg-acp (bkg-protocol)

**ACP JSON-RPC 2.0. AgentBridge. 24 methods.**

The Agent Communication Protocol is the typed interface between
external agents and the BKG runtime.

## 24 ACP Methods

| Category | Methods |
|---|---|
| Session | `session.create`, `session.end`, `session.status` |
| Messages | `message.send`, `message.stream`, `message.history` |
| Tasks | `task.create`, `task.update`, `task.complete`, `task.list` |
| Approvals | `approval.request`, `approval.decide`, `approval.list` |
| Tools | `tool.call`, `tool.result`, `tool.list` |
| Agents | `agent.spawn`, `agent.stop`, `agent.status` |
| Events | `event.subscribe`, `event.unsubscribe`, `event.replay` |
| Capabilities | `cap.grant`, `cap.revoke`, `cap.list` |

## Invariant

All ACP calls produce typed events in the ledger.
AgentBridge translates stdout → `UniversalEvent` stream.
