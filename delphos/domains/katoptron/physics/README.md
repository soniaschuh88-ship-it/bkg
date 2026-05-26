# bkg-physics

**Deterministic n-body physics for DAG layout.**

Forces model task dependencies as physical springs.
High entropy = chaotic system. Low entropy = stable.

## Key Types

| Type | Purpose |
|---|---|
| `PhysicsNode` | `{ mass, x, y, vx, vy, pinned }` |
| `SpringForce` | Edge tension (dependency criticality) |
| `GravityForce` | Node attraction |
| `PhysicsSimulation` | Runs to convergence |
| `system_entropy()` | `blocked_ratio * 0.6 + edge_density * 0.4` |

## Convergence

Simulation converges when `max_velocity < 0.01`.
Pinned nodes never move.
