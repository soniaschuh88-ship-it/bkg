//! # bkg-workflow — Workflow Gates: Plan→Review→Execute
//! Single source of truth for all workflow execution in DELPHOS.
pub mod engine; pub mod gate; pub mod graph; pub mod phase; pub mod verdict;
pub use engine::WorkflowEngine;
pub use gate::{WorkflowGate, GateConfig};
pub use graph::ExecutionGraph;
pub use phase::WorkflowPhase;
pub use verdict::{Verdict, VerdictRecord};