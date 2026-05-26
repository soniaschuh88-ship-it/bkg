//! bkg-snapshot — global world snapshots: fork, export, restore.
//! Single source of truth.
pub mod reality; pub mod realm; pub mod timeline;
pub use reality::{RealitySnapshot, SnapshotId};
pub use realm::RealmSnapshot;
pub use timeline::TimelineSnapshot;
