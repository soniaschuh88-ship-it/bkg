# bkg-vm

**Tool sandbox VM. Deterministic execution environment.**

All tool calls run inside a `SandboxVm`. File access requires `VfsMount`.
Resource limits enforced. Snapshots enable rollback.

## Key Types

| Type | Purpose |
|---|---|
| `SandboxVm` | Isolated execution context |
| `ResourceLimits` | `max_memory_mb`, `max_cpu_percent`, `max_time_secs` |
| `VfsMount` | Scoped filesystem access (ReadOnly/ReadWrite/Hidden) |
| `SyscallFilter` | Allowlist of permitted syscalls |
| `VmSnapshot` | Point-in-time state for rollback |

## Presets

```rust
ResourceLimits::strict()      // 128MB, 25% CPU, 60s
ResourceLimits::default()     // 512MB, 50% CPU, 300s
ResourceLimits::permissive()  // 4GB, 100% CPU, 3600s
```
