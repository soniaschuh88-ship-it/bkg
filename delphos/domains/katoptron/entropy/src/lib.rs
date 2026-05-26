//! bkg-entropy — system entropy, pressure, heat, stability metrics.
//! Telemetry physics: system properties as physical observables.
//! Single source of truth.
pub mod heat; pub mod metrics; pub mod pressure; pub mod stability;
pub use metrics::{SystemMetrics, MetricSnapshot};
