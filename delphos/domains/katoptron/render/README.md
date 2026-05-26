# bkg-render

**Render backends. Terminal, ANSI, headless.**

Backend abstraction over the `bkg-compiler` bytecode output.
Headless backend enables CI screenshot testing.

## Backends

| Backend | Use case |
|---|---|
| `HeadlessBackend` | CI + snapshot testing (pixel buffer) |
| ANSI | Terminal escape codes |
| ratatui | TUI widgets (planned) |
| WebGPU | Browser (planned) |

## Invariant

All backends implement `RenderBackend`.
Headless backend is deterministic: same bytecode = same pixel buffer.
