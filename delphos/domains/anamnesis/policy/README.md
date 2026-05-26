# bkg-policy

**PolicyEngine. Cross-cutting governance rules.**

Policies apply to all realms. They are evaluated before any state change.
Policy violations halt execution.

## Key Types

| Type | Purpose |
|---|---|
| `PolicyEngine` | Evaluates policies against proposed actions |
| `Policy` | Named rule with condition + consequence |
| `PolicyResult` | `Allow / Deny(reason) / Audit` |
