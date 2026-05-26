//! # bkg-task — Task Capsules + Lifecycle + DAG
//! Single source of truth for all task concepts in DELPHOS.
//! Task capsules: .bkg/tasks/{id}/ filesystem layout.
pub mod capsule; pub mod dag; pub mod lifecycle; pub mod task;
pub use dag::{DependencyGraph, DagError};
pub use lifecycle::{TaskStatus, TaskTransitionError};
pub use task::{Task, TaskId, TaskPriority};