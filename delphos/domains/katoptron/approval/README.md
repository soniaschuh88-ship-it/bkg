# bkg-approval

**Approval gates. Immutable audit trail.**

All dangerous operations require approval.
The audit trail is append-only and cryptographically linked.

## Key Types

| Type | Purpose |
|---|---|
| `ApprovalGate` | `pending → approved/rejected` |
| `ApprovalRequest` | `{ kind, description, context, risk }` |
| `ApprovalAudit` | Append-only audit event log |
| `ActionPolicy` | `allow / block / require-approval` |

## ApprovalKind

`Merge`, `DangerousToolUse`, `BudgetOverrun`, `AgentSpawn`, `SecretAccess`, `Custom`

## Invariant

Double-decide is rejected (idempotency).
Every decision is recorded in the immutable audit trail.
