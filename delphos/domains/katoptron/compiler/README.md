# bkg-compiler

**Katoptron UI compiler pipeline. RealmState → Bytecode.**

Deterministic: same AST → same Bytecode always.
Backend-agnostic bytecode runs on ANSI, ratatui, WebGPU, headless.

## Pipeline

```
RealmState → UiAst → GeometryGraph → Bytecode → UiFrame
```

## Key Types

| Type | Purpose |
|---|---|
| `UiAst` | Abstract UI tree |
| `UiNode` | `Panel / Card / Text / Badge / Button / Row / Column` |
| `UiCompiler` | Pure function: `UiAst → Bytecode` |
| `Bytecode` | Portable render instructions |
| `UiFrame` | Rendered output for one tick |

## Invariant

`UiCompiler::compile()` is a pure function. Same AST = same Bytecode. Always.
