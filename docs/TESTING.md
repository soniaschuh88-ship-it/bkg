# BKG — Testing Guide

---

## Test Philosophy

Tests in BKG are not just correctness checks.
They are **formal proofs embedded in code**.

Examples:
- `delta_is_total_never_panics` — calls all 522 (phase, input) combinations
- `canonical_algebra_agrees_with_kernel_everywhere` — 100% alignment verification
- `causal_importance_sums_to_one` — information partition invariant
- `all_phases_reachable_from_genesis` — structural completeness

---

## Running Tests

```bash
# Full workspace (all 400+ tests)
cargo test --workspace

# Single crate
cargo test -p bkg-kernel

# Specific test
cargo test -p bkg-kernel -- semantic_weight::tests::causal_importance_sums_to_one

# With output
cargo test -p bkg-kernel -- --nocapture

# Library only (no doc tests)
cargo test --lib -p bkg-kernel
```

---

## Test Categories

### Unit tests
Per-module behavior. Live in `#[cfg(test)] mod tests { ... }` at bottom of each file.

### Integration tests
Cross-crate pipeline proofs. Live in `cognition/kernel/src/integration.rs`.

### Formal verification tests
Exhaustive enumeration over finite domains.

```rust
#[test]
fn delta_is_total_never_panics() {
    // Calls all 18 × 29 = 522 combinations
    for &phase in KernelPhase::ALL {
        for &input in KernelInputKind::ALL {
            let _ = kernel_delta(phase, input);
        }
    }
}
```

### Invariant tests
Check that structural invariants hold.

```rust
#[test]
fn canonical_algebra_agrees_with_kernel_everywhere() {
    let algebra = canonical_constraint_rules();
    let events = DriftDetector::check_algebra_vs_kernel(&algebra);
    assert!(events.iter().filter(|e| e.severity() == DriftSeverity::Critical).count() == 0);
}
```

---

## Test Counts by Crate (key crates)

| Crate | Tests | Key property tested |
|---|---|---|
| `bkg-kernel` | 231 | L0–L12 formal stack, all 522 δ cells |
| `bkg-state` | 33 | Projection isolation, contract, rebuild |
| `bkg-event` | 20 | TypedEvent<P>, 9 canonical types |
| `bkg-enforce` | 15 | Sealed traits, invariant guards |
| `bkg-ecs` | 18 | Stable iteration, generation IDs |

---

## Determinism Requirements

All tests must be deterministic:

```rust
// WRONG — non-deterministic
let id = Uuid::new_v4().to_string();

// RIGHT — deterministic (via DeterministicId)
let id = DeterministicId::derive("seed", &["task", "T-001"], RealmId::Telum);
```

No test may use `SystemTime::now()` for ordering.
No test may use `rand` without a fixed seed.

---

## Checking for Drift

After any kernel change, run the drift detector:

```bash
cargo test -p bkg-kernel -- specification_drift
```

This verifies:
- Algebra agrees with kernel on all 522 cells
- No rule conflicts
- Pipeline is acyclic

If `canonical_algebra_agrees_with_kernel_everywhere` fails → fix immediately.
