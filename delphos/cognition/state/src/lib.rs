//! # bkg-state — RealmStateMachine
//! The single canonical state reducer for DELPHOS.
//! Single source of truth. One module, one location.
//!
//! **The only legal state mutation path:**
//! ```text
//! DomainEvent → Reducer::apply(&RealmState, event) → Result<RealmState>
//! ```
//! No crate may mutate `RealmState` directly. Zero mutable globals. Copy-on-write.

pub mod invariants;
pub mod mutation;
pub mod projection; pub mod projection_contract; pub mod projection_view;
pub mod realm_state;
pub mod reconciliation;
pub mod reducer;
pub mod snapshot;
pub mod transition;

pub use projection::{Projection, ProjectionId};
pub use realm_state::RealmState;
pub use reducer::{Reducer, ReducerId};
pub use snapshot::StateSnapshot;
pub use transition::{StateTransition, TransitionError};

pub use projection_contract::{EventRange, KernelStamp, MaterializerKernel, ProjectionChecksum, ProjectionContract, ProjectionVersion, ProjectionViolation, RebuildLog, RebuildProof};
pub use projection_view::{ProjectionFactory, AgentStatusProjection, KanbanProjection, ProjectionView, TaskListProjection, TaskSummary};
