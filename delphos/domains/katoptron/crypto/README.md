# bkg-crypto

**BLAKE3 hashing. Ed25519 signatures. Cryptographic primitives.**

The single source of truth for all cryptographic operations.
No other crate may call crypto libraries directly.

## Invariant

- All hashes in the workspace use BLAKE3 (not SHA256, not MD5)
- All signatures use Ed25519 (not RSA, not ECDSA)
- Keys are never stored in plaintext — use `bkg-secrets`
