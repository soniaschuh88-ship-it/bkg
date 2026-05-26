//! bkg-vm — deterministic tool execution sandbox.
//! Syscall layer, VFS mounts, resource limits, snapshot rollback.
//! Single source of truth.
pub mod limits; pub mod mount; pub mod process; pub mod snapshot; pub mod syscall; pub mod vm;
pub use limits::ResourceLimits;
pub use mount::{VfsMount, MountPolicy};
pub use process::{VmProcess, ProcessResult};
pub use snapshot::VmSnapshot;
pub use vm::{SandboxVm, VmError};
