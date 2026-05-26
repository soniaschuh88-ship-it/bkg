//! bkg-approval — approval gates + immutable audit trail.
//! Single source of truth for all approval flows in DELPHOS.
pub mod audit; pub mod gate; pub mod policy; pub mod request;
pub use gate::{ApprovalGate, GateStatus};
pub use policy::ActionPolicy;
pub use request::{ApprovalRequest, ApprovalKind, ApprovalResponse};
pub use audit::ApprovalAudit;
