//! bkg-lanes — Realm Bus IPC fabric.
//! Deterministic inter-realm transport. Single source of truth.
pub mod backpressure; pub mod bus; pub mod lane; pub mod packet; pub mod qos; pub mod router;
pub use bus::{BusError, RealmBus};
pub use lane::{Lane, LaneClass};
pub use packet::{BusPacket, PacketId, PacketStatus};
pub use router::LaneRouter;
