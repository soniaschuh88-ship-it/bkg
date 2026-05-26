# bkg-core

**Primitive types, IDs, errors. The foundation of every other crate.**

Single source of truth for `RealmId`, `Hash256`, `BkgError`, `BkgResult`.

## Key Types

| Type | Purpose |
|---|---|
| `RealmId` | Strongly-typed realm identifier (enum) |
| `Hash256` | 32-byte hash — BLAKE3 or similar |
| `BkgError` | The single error type for the workspace |
| `BkgResult<T>` | `Result<T, BkgError>` alias |

## Invariants

- No crate may define its own error type — use `BkgError`
- All cross-realm references use `RealmId`, never raw strings
- `Hash256` is the only hash representation in the workspace

## Realms

```rust
pub enum RealmId {
    Telum, Katoptron, Styx, Causa, Arche, Speculum,
    Anamnesis, Mnemos, Thalassa,
}
```
