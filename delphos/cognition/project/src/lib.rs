//! # bkg-project — Project Registry + Settings
//! Single source of truth for all project configuration in DELPHOS.
pub mod project; pub mod registry; pub mod settings;
pub use project::{Project, ProjectId};
pub use registry::ProjectRegistry;
pub use settings::{GlobalSettings, ProjectSettings};