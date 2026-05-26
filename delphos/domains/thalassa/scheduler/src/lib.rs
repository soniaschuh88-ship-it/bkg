//! # bkg-scheduler — Deterministic DAG Scheduler
//! Single source of truth for task scheduling in DELPHOS.
pub mod lease; pub mod priority; pub mod scheduler;
pub use priority::SchedulerPriority;
pub use scheduler::{TaskScheduler, ScheduledTask, SchedulerError};