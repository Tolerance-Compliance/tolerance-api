//! SEP-2243 header/body validation for the Streamable HTTP transport.

use axum::http::HeaderMap;
use serde_json::Value;

use super::constants::{HEADER_MCP_METHOD, HEADER_MCP_NAME, HEADER_MCP_PROTOCOL_VERSION};
use super::protocol::JsonRpcRequest;

/// Validate the SEP-2243 routable headers (`Mcp-Method`, `Mcp-Name`) and
/// `MCP-Protocol-Version` against the request body. Returns a rejection
/// message on mismatch, `None` when consistent (or when headers are absent —
/// legacy clients don't send them).
pub fn header_body_mismatch(headers: &HeaderMap, request: &JsonRpcRequest) -> Option<String> {
    if let Some(h) = header_str(headers, HEADER_MCP_METHOD)
        && h != request.method
    {
        return Some(format!(
            "HeaderMismatch: Mcp-Method header '{h}' does not match body method '{}'",
            request.method
        ));
    }

    if request.method == "tools/call"
        && let Some(h) = header_str(headers, HEADER_MCP_NAME)
    {
        let body_name = request
            .params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if h != body_name {
            return Some(format!(
                "HeaderMismatch: Mcp-Name header '{h}' does not match body tool name '{body_name}'"
            ));
        }
    }

    // Protocol version: header must match body `_meta.protocolVersion` when
    // both are present (modern clients send both).
    if let Some(h) = header_str(headers, HEADER_MCP_PROTOCOL_VERSION) {
        let body_version = request
            .params
            .as_ref()
            .and_then(|p| p.get("_meta"))
            .and_then(|m| m.get("protocolVersion"))
            .and_then(Value::as_str);
        if let Some(bv) = body_version
            && h != bv
        {
            return Some(format!(
                "HeaderMismatch: MCP-Protocol-Version header '{h}' does not match _meta.protocolVersion '{bv}'"
            ));
        }
    }

    None
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}
