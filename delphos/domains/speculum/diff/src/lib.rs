pub mod capsule_diff; pub mod graph_diff; pub mod state_diff; pub mod trace;
pub use state_diff::{StateDiff, DiffEntry, DiffKind};
pub use trace::CausalTrace;
