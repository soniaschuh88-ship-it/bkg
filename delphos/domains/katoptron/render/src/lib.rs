//! bkg-render — render backend abstraction.
//! Backends: terminal (ratatui), ANSI, headless (CI/tests), WebGPU stub.
//! Single source of truth.
pub mod ansi; pub mod backend; pub mod headless;
pub use backend::{RenderBackend, RenderOutput};
pub use headless::HeadlessBackend;
