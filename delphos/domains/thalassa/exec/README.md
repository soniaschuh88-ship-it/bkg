# bkg-exec

**Deterministic tool executors: bash, file, grep, glob.**

All tool calls are recorded in the event ledger.
Results are deterministic given the same inputs.

## Executors

- `BashExecutor` — shell command execution with timeout
- `FileExecutor` — read/write/list within allowed paths
- `GrepExecutor` — pattern search with line numbers
- `GlobExecutor` — path pattern matching

## Invariant

All executors produce typed `ToolResult` events.
No executor mutates state directly — all mutations go through events.
