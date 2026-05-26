# bkg-secrets

**AES-256-GCM encrypted secrets. OS keychain integration.**

Secrets are never stored in plaintext. Access is audited.
`materialize_env()` exports secrets as environment variables for tool execution.

## Key Types

| Type | Purpose |
|---|---|
| `SecretsStore` | Central secret store |
| `Secret` | Encrypted value + scope + policy |
| `SecretScope` | `Project(id) / Global` |
| `AccessPolicy` | `Auto / Prompt / Deny` |

## Invariant

- `AccessPolicy::Deny` → decrypt() returns None (structural, not runtime check)
- All reads are logged with reader identity
- Project-scoped secrets are completely isolated from each other
