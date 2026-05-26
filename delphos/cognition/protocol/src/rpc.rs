//! JSON-RPC 2.0 types. Single source of truth for all ACP wire messages.
//! Ported from sandbox-agent acp-http-adapter JSON-RPC layer.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 request id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcId { Str(String), Num(i64), Null }

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<RpcId>,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl RpcRequest {
    pub fn new(method: impl Into<String>, params: impl Serialize) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(RpcId::Str(uuid::Uuid::new_v4().to_string())),
            method: method.into(),
            params: Some(serde_json::to_value(params).unwrap_or(Value::Null)),
        }
    }
    pub fn notification(method: impl Into<String>, params: impl Serialize) -> Self {
        Self { jsonrpc: "2.0".into(), id: None, method: method.into(),
               params: Some(serde_json::to_value(params).unwrap_or(Value::Null)) }
    }
    pub fn is_notification(&self) -> bool { self.id.is_none() }
}

/// Standard JSON-RPC 2.0 error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    Custom(i32),
}

impl RpcErrorCode {
    pub fn code(self) -> i32 {
        match self {
            Self::ParseError     => -32700,
            Self::InvalidRequest => -32600,
            Self::MethodNotFound => -32601,
            Self::InvalidParams  => -32602,
            Self::InternalError  => -32603,
            Self::Custom(c)      => c,
        }
    }
    pub fn message(self) -> &'static str {
        match self {
            Self::ParseError     => "Parse error",
            Self::InvalidRequest => "Invalid Request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams  => "Invalid params",
            Self::InternalError  => "Internal error",
            Self::Custom(_)      => "Server error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn new(code: RpcErrorCode, data: Option<Value>) -> Self {
        Self { code: code.code(), message: code.message().to_string(), data }
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self { code: RpcErrorCode::InternalError.code(), message: msg.into(), data: None }
    }
    pub fn method_not_found(method: &str) -> Self {
        Self { code: RpcErrorCode::MethodNotFound.code(), message: format!("method '{method}' not found"), data: None }
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    pub fn success(id: Option<RpcId>, result: impl Serialize) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(serde_json::to_value(result).unwrap_or(Value::Null)), error: None }
    }
    pub fn error(id: Option<RpcId>, error: RpcError) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: None, error: Some(error) }
    }
    pub fn is_success(&self) -> bool { self.error.is_none() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn request_roundtrip() {
        let req = RpcRequest::new("session/create", serde_json::json!({"agent":"claude"}));
        let json = serde_json::to_string(&req).unwrap();
        let back: RpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.method, "session/create");
        assert!(!back.is_notification());
    }
    #[test] fn notification_no_id() {
        let n = RpcRequest::notification("session/event", serde_json::json!({}));
        assert!(n.is_notification());
    }
    #[test] fn error_codes() {
        assert_eq!(RpcErrorCode::MethodNotFound.code(), -32601);
        let err = RpcError::internal("boom");
        assert_eq!(err.code, -32603);
    }
    #[test] fn response_success() {
        let r = RpcResponse::success(Some(RpcId::Num(1)), serde_json::json!({"ok":true}));
        assert!(r.is_success());
    }
}