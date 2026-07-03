//! The `POST /mcp` axum handler: SEP-2243 header validation + JSON-RPC
//! method dispatch. Stateless — every request is self-contained.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

use crate::cmmc::state::CmmcState;

use super::protocol::{INVALID_PARAMS, INVALID_REQUEST, JsonRpcRequest, JsonRpcResponse};
use super::tools;

/// The stateless protocol revision this server is built for.
const MODERN_PROTOCOL_VERSION: &str = "2026-07-28";

/// Handshake-era protocol revisions we answer `initialize` for.
const LEGACY_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"];

/// Fallback version offered when a legacy client requests something unknown;
/// per spec the client decides whether it can proceed.
const LEGACY_FALLBACK_VERSION: &str = "2025-11-25";

const SERVER_INSTRUCTIONS: &str = "Read-only tools over the official NIST SP 800-53/171/171A/172 \
     and FAR 52.204-21 corpora plus CMMC scoring and POA&M eligibility rules. Start with \
     list_documents to discover valid document/revision arguments. Results are TOON-encoded text.";

/// Handle one MCP request. Notifications get `202 Accepted` with no body;
/// requests get a JSON-RPC response object (JSON only — no SSE, every tool
/// answers immediately from memory).
pub async fn handle_mcp(
    State(state): State<CmmcState>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let request: JsonRpcRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            let resp =
                JsonRpcResponse::error(None, INVALID_REQUEST, format!("Invalid JSON-RPC: {e}"));
            return (StatusCode::BAD_REQUEST, Json(resp)).into_response();
        }
    };

    // SEP-2243: when the routable headers are present they MUST match the
    // body. We don't *require* them so legacy (handshake-era) clients keep
    // working, but a mismatch is always a hard reject.
    if let Some(mismatch) = header_body_mismatch(&headers, &request) {
        let resp = JsonRpcResponse::error(request.id, INVALID_REQUEST, mismatch);
        return (StatusCode::BAD_REQUEST, Json(resp)).into_response();
    }

    // Notifications (no id) require no response body.
    if request.id.is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    let JsonRpcRequest {
        id, method, params, ..
    } = request;
    let params = params.unwrap_or(Value::Null);

    let response = match method.as_str() {
        "initialize" => JsonRpcResponse::success(id, initialize_result(&params)),
        "server/discover" => JsonRpcResponse::success(id, discover_result()),
        "ping" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => JsonRpcResponse::success(id, json!({ "tools": tools::definitions() })),
        "tools/call" => match params.get("name").and_then(Value::as_str) {
            Some(name) => {
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let result = tools::call(&state, name, &args);
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or_default())
            }
            None => JsonRpcResponse::error(id, INVALID_PARAMS, "Missing tool 'name' in params"),
        },
        _ => JsonRpcResponse::method_not_found(id),
    };

    Json(response).into_response()
}

/// Legacy `initialize` (pre-2026-07-28): echo a mutually supported version.
fn initialize_result(params: &Value) -> Value {
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(LEGACY_FALLBACK_VERSION);
    let negotiated =
        if LEGACY_PROTOCOL_VERSIONS.contains(&requested) || requested == MODERN_PROTOCOL_VERSION {
            requested
        } else {
            LEGACY_FALLBACK_VERSION
        };

    json!({
        "protocolVersion": negotiated,
        "capabilities": capabilities(),
        "serverInfo": server_info(),
        "instructions": SERVER_INSTRUCTIONS,
    })
}

/// Modern `server/discover` (2026-07-28): capability discovery without a
/// handshake or session.
fn discover_result() -> Value {
    json!({
        "protocolVersion": MODERN_PROTOCOL_VERSION,
        "capabilities": capabilities(),
        "serverInfo": server_info(),
        "instructions": SERVER_INSTRUCTIONS,
    })
}

fn capabilities() -> Value {
    json!({ "tools": { "listChanged": false } })
}

fn server_info() -> Value {
    json!({ "name": "tolerance-api", "version": env!("CARGO_PKG_VERSION") })
}

/// Validate the SEP-2243 routable headers and `MCP-Protocol-Version` against
/// the request body. Returns a rejection message on mismatch.
fn header_body_mismatch(headers: &HeaderMap, request: &JsonRpcRequest) -> Option<String> {
    if let Some(h) = header_str(headers, "mcp-method")
        && h != request.method
    {
        return Some(format!(
            "HeaderMismatch: Mcp-Method header '{h}' does not match body method '{}'",
            request.method
        ));
    }

    if request.method == "tools/call"
        && let Some(h) = header_str(headers, "mcp-name")
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
    if let Some(h) = header_str(headers, "mcp-protocol-version") {
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
