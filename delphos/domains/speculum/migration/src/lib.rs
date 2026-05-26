//! bkg-migration — replay-safe schema migration runner.
//! Single source of truth for all schema version upgrades.
pub mod plan; pub mod runner; pub mod version_map;
pub use plan::{MigrationPlan, MigrationStep};
pub use runner::{MigrationRunner, MigrationOutcome};
pub use version_map::VersionMap;
