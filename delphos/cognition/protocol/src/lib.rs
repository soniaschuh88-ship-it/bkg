//! # bkg-acp  (cognition/protocol)
//!
//! BKG Agent Control Protocol — JSON-RPC 2.0 proxy and agent process bridge.
//!
//! **Rebranded from** sandbox-agent `acp-http-adapter` and `acp-proxy-runtime`.
//!
//! **Single source of truth — one module, one location.**
//!
//! ## What lives here
//! - `RpcRequest` / `RpcResponse` — JSON-RPC 2.0 types
//! - `AcpMethod` — all BKG ACP methods
//! - `AgentBridge` — spawns and manages agent processes
//! - `InferenceProxy` — routes requests through bkg-providers fallback chain
//!
//! ## ACP method namespace
//! Custom extension methods use the `_bkg/` prefix (replaces `_sandboxagent/`).

pub mod bridge;
pub mod inference;
pub mod method;
pub mod rpc;

pub use bridge::{AgentBridge, BridgeConfig, BridgeEvent};
pub use inference::{InferenceProxy, InferenceRequest, InferenceResponse};
pub use method::AcpMethod;
pub use rpc::{RpcError, RpcErrorCode, RpcId, RpcRequest, RpcResponse};