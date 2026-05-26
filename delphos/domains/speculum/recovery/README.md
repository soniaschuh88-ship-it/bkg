# bkg-recovery

**Crash reconstruction. Partial replay repair.**

When the system crashes mid-write, recovery finds the last good checkpoint
and replays from there.

## Key Types

| Type | Purpose |
|---|---|
| `CrashClassification` | `HashChainBroken / CapsuleCorrupted / ReplayDiverged / PartialWrite / MeshDesync` |
| `RepairStrategy` | `RollbackToSnapshot / ReplayFromLastGoodEvent / RequestManualIntervention` |
| `RecoveryCheckpoint` | `{ realm_id, event_id, lamport, state_checksum }` |

## Strategy Selection

```
HashChainBroken   → RollbackToSnapshot
CapsuleCorrupted  → RollbackToSnapshot
PartialWrite      → ReplayFromLastGoodEvent
MeshDesync        → ReplayFromLastGoodEvent
ReplayDiverged    → RequestManualIntervention (cannot auto-recover)
```
