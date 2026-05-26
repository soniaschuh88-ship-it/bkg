# bkg-simulation

**Deterministic execution simulator. Test without real agents.**

Run workflows and policies against a fake world.
Oracle assertions verify correctness without side effects.

## Key Types

| Type | Purpose |
|---|---|
| `SimWorld` | Deterministic fake world with entities + events |
| `SimAgent` | Scriptable fake agent (action sequence) |
| `Oracle` | Assertion engine for simulation verification |
| `SimTick` | Monotone tick counter |

## Usage

```rust
let mut world = SimWorld::new();
let agent = SimAgent::new("test").with_action("create task T-1");
agent.run(&mut world);
oracle.assert_entity_exists(&world, "T-1", "task created");
assert!(oracle.all_passed());
```
