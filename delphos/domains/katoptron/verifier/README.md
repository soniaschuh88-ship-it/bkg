# bkg-verifier

**Hash-chain verification. PermissionEnforcer. Drift detection.**

Verifies the integrity of the event ledger and capsule state.
Any hash mismatch = immediate halt.

## Key Types

| Type | Purpose |
|---|---|
| `HashChainVerifier` | Walks the ledger chain, detects breaks |
| `PermissionEnforcer` | `ReadOnly / WorkspaceWrite / DangerFullAccess` |
| `DriftChecker` | Detects state divergence between nodes |

## Invariant

Hash-chain break → system halt. No silent corruption.
