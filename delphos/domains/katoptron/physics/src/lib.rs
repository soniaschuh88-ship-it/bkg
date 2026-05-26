//! bkg-physics — DAG physics engine: node mass, edge tension, entropy.
//! Deterministic n-body layout for UI visualization.
//! Single source of truth.
pub mod entropy; pub mod forces; pub mod layout; pub mod node; pub mod simulation;
pub use node::{PhysicsNode, NodeId};
pub use simulation::{PhysicsSimulation, SimState};
