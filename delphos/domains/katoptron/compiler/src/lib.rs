//! bkg-compiler — Katoptron UI compiler pipeline.
//! EventLedger → RealmState → UiAst → Geometry → Bytecode → Frame.
//! Single source of truth.
pub mod ast; pub mod bytecode; pub mod compiler; pub mod frame; pub mod geometry;
pub use ast::{UiAst, UiNode, UiNodeKind};
pub use bytecode::{Bytecode, BytecodeOp};
pub use compiler::UiCompiler;
pub use frame::UiFrame;
