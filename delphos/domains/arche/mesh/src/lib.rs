//! bkg-mesh — multi-node replication, mDNS discovery, lease management.
//! Single source of truth for all mesh topology in DELPHOS.
pub mod discovery; pub mod health; pub mod lease; pub mod node; pub mod sync;
pub use node::{MeshNode, MeshNodeId, NodeStatus};
pub use lease::{MeshLease, LeaseError};
pub use sync::{SyncRecord, SyncStatus};
pub use discovery::NodeRegistry;
pub use health::NodeHealth;
