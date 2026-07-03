//! JSON-RPC 2.0 envelope types and error codes
//! (https://www.jsonrpc.org/specification#error_object).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;

/// Incoming JSON-RPC 2.0 request (or a *notification* when `id` is absent).
#[derive(Debug, Deserialize, ToSchema)]
pub struct JsonRpcRequest {
    /// Protocol version. Always `"2.0"`.
    #[allow(dead_code)]
    #[schema(example = "2.0")]
    pub jsonrpc: String,
    /// Correlation id echoed back in the response. Omit it to send a
    /// *notification* (fire-and-forget; the server replies `202 Accepted`
    /// with no body).
    #[schema(value_type = Object, example = json!(1))]
    pub id: Option<Value>,
    /// Method to invoke: `initialize`, `server/discover`, `ping`,
    /// `tools/list`, or `tools/call`.
    #[schema(example = "tools/call")]
    pub method: String,
    /// Method parameters. For `tools/call` this is
    /// `{ "name": "<tool>", "arguments": { … } }`.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub params: Option<Value>,
}

/// Outgoing JSON-RPC 2.0 response. Exactly one of `result` (success) or
/// `error` (failure) is present.
#[derive(Debug, Serialize, ToSchema)]
pub struct JsonRpcResponse {
    /// Protocol version. Always `"2.0"`.
    #[schema(value_type = String, example = "2.0")]
    pub jsonrpc: &'static str,
    /// Echoes the request `id` (absent for a response to a notification).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, example = json!(1))]
    pub id: Option<Value>,
    /// The method's result payload, present on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub result: Option<Value>,
    /// The error object, present on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Serialize, ToSchema)]
pub struct JsonRpcError {
    /// Error code: `-32600` invalid request, `-32601` method not found,
    /// `-32602` invalid params.
    #[schema(example = -32601)]
    pub code: i64,
    /// Human-readable error message.
    #[schema(example = "Method not found")]
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }

    pub fn method_not_found(id: Option<Value>) -> Self {
        Self::error(id, METHOD_NOT_FOUND, "Method not found")
    }
}
