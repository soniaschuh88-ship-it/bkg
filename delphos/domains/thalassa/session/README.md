# bkg-session

**UniversalEvent (10 types). BkgSession broadcast + replay.**

Sessions are the runtime context for agent execution.
All events are typed via `UniversalEvent`. Sessions can be replayed.

## UniversalEvent Types (10)

`AgentStarted`, `AgentStopped`, `MessageSent`, `MessageReceived`,
`ToolCall`, `ToolResult`, `ApprovalRequired`, `ApprovalDecided`,
`ErrorOccurred`, `SessionCompleted`

## Key Types

| Type | Purpose |
|---|---|
| `BkgSession` | Active session with broadcast + replay |
| `UniversalMessage` | 8 message parts (text, code, tool, approval, ...) |
| `SessionManager` | Lifecycle + persistence |
