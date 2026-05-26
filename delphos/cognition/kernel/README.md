# bkg-kernel

**The formal kernel. M=(Q,Σ,Λ,δ,λ,q₀). 25 modules. L0–L12. 231 tests.**

The law of physics for DELPHOS. Every event must pass through here.
No bypass. No shortcuts. No exceptions.

See [`/docs/KERNEL_FORMAL_SYSTEM.md`](/docs/KERNEL_FORMAL_SYSTEM.md) for the complete specification.

## Key Modules

| Module | Layer | Purpose |
|---|---|---|
| `kernel_state.rs` | L0 | KernelPhase(18), KernelInputKind(29), kernel_delta |
| `constraint_algebra.rs` | L0 | Symbolic predicates, rule synthesis |
| `kernel_machine.rs` | L1 | KernelMachine runner, TransitionRecord |
| `proof_certificate.rs` | L2 | ExecutionTrace, ProofChecker (~30 lines trusted core) |
| `trace_synthesizer.rs` | L3 | Inductive rule synthesis from traces |
| `specification_drift.rs` | L4 | Triple-layer drift detection |
| `specification_entropy.rs` | L5 | Shannon + Gini + structural diversity |
| `algebra_stability.rs` | L6 | AlgebraInvariant, PinnedRuleSet, SynthesisCycleGuard |
| `semantic_weight.rs` | L7 | NecessityClass, causal importance, composite weight |
| `rule_simplifier.rs` | L8 | Safe Remove/Merge/Generalize |
| `counterfactual.rs` | L9/L10 | BFS reachability, CounterfactualWitness, SemanticFixationGuard |
| `counterfactual_competition.rs` | L11 | DomainInterference, UniqueCriticalCoverage |
| `semantic_growth.rs` | L12 | Expressiveness Conservation Law |
| `pipeline.rs` | — | EventPipeline (validate→decide→apply→emit) |
| `state_transition.rs` | — | StateTransitionFn<E>, ReplayIdentityProof |
| `event_ledger.rs` | — | Append-only, BLAKE3 hash-chained |
| `realm.rs` | — | Realm (atomic commit, zero dual-truth drift) |
| `integration.rs` | — | Compile-time pipeline proof |

## The Pipeline

```
EventIn → ABI → Schema → Clock → Capability → Causal → Decide → Apply → Stamp → Emit
```

## Empirical State (canonical spec)

- 18 KernelPhases, 29 KernelInputKinds = 522 cells
- 20 constraint rules cover the full table
- 80.1% semantic space free (418/522 cells)
- All 18 phases reachable from Genesis (max distance = 11)
