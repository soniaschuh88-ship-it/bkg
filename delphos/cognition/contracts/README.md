# bkg-contracts

**CausalContract. The permission system for cross-realm events.**

A `CausalContract` explicitly permits one realm to send events to another.
Without a contract, cross-realm events are rejected by the kernel.

## Key Types

| Type | Purpose |
|---|---|
| `CausalContract` | Bidirectional permission grant |
| `ContractId` | Unique contract identifier |
| `ContractScope` | Which event types are permitted |

## Invariant

No cross-realm event is processed without a matching `CausalContract`.
The `KernelArbitrator` validates every contract on receipt.
