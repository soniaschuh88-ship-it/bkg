# bkg-mesh

**Multi-node replication. Lease management. mDNS discovery.**

Nodes join the mesh, acquire leases on resources, and sync state.
Epoch fencing prevents split-brain.

## Key Types

| Type | Purpose |
|---|---|
| `MeshNode` | `{ id, label, address, status, lamport }` |
| `LeaseRegistry` | Epoch-fenced lease acquisition + recovery |
| `MeshLease` | `{ resource_id, holder, epoch, expires_at }` |
| `NodeRegistry` | Active node discovery + staleness detection |
| `NodeHealth` | Heartbeat age + resource metrics |
| `SyncRecord` | State sync with checksum + status |

## Lease Invariant

Higher epoch always wins. Same epoch → contested (rejected).
Abandoned leases (expired) are recovered automatically.
