//! JSON-RPC 2.0 + MCP wire types (the small subset a stateless tool server
//! needs). Adapted from the `scherso/axum-mcp` skeleton.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Incoming JSON-RPC request or notification (no `id` ⇒ notification).
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// Outgoing JSON-RPC response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

/// JSON-RPC error codes (https://www.jsonrpc.org/specification#error_object).
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;

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

/// A tool surfaced via `tools/list`.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// One piece of content in a tool result (text only — our results are TOON).
#[derive(Debug, Clone, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: String,
}

impl Content {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            kind: "text",
            text: text.into(),
        }
    }
}

/// The result of `tools/call`.
#[derive(Debug, Clone, Serialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(rename = "isError", skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl CallToolResult {
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: false,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(message)],
            is_error: true,
        }
    }
}
