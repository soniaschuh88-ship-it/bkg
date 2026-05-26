# bkg-capabilities

**Realm-scoped permission tokens. Prevents agents from becoming all-powerful.**

Every capability is a named, time-bounded, revocable grant.
`ExecutionScope` is the signed context passed into tool invocations.

## Key Types

| Type | Purpose |
|---|---|
| `CapabilitySet` | BTreeSet of capability IDs (sorted, deterministic) |
| `CapabilityGrant` | TTL + revocable + signed |
| `ExecutionScope` | Signed scope for sandboxed tool execution |

## Well-known Capabilities

```
files:read        files:write        bash:execute
network:outbound  agent:spawn        secrets:access
```

## Invariant

Expired or revoked grants: `is_active() = false` → `has(cap) = false`.
No capability can be granted without going through `ApprovalGate`.
