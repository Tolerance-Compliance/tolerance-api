//! The `POST /mcp` axum handler: parse, validate, dispatch. Stateless —
//! every request is self-contained. Header validation lives in
//! [`super::headers`], handshake/discovery responses in [`super::discovery`].

use axum::http::HeaderMap;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

use crate::cmmc::state::CmmcState;

use super::discovery;
use super::headers;
use super::protocol::{INVALID_PARAMS, INVALID_REQUEST, JsonRpcRequest, JsonRpcResponse};
use super::tools;

/// Handle one MCP request. Notifications get `202 Accepted` with no body;
/// requests get a JSON-RPC response object (JSON only — no SSE, every tool
/// answers immediately from memory).
pub async fn handle_mcp(
    State(state): State<CmmcState>,
    header_map: HeaderMap,
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
    if let Some(mismatch) = headers::header_body_mismatch(&header_map, &request) {
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
        "initialize" => JsonRpcResponse::success(id, discovery::initialize_result(&params)),
        "server/discover" => JsonRpcResponse::success(id, discovery::discover_result()),
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
