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

/// Model Context Protocol endpoint (JSON-RPC 2.0)
///
/// A single endpoint that exposes the same NIST/FAR corpora the REST API
/// serves as **MCP tools**, so an LLM agent (Claude, Cursor, …) can query the
/// standards directly. It speaks [JSON-RPC 2.0](https://www.jsonrpc.org/specification):
/// POST one request object, get one response object back. It is stateless and
/// read-only, with the same public exposure as the REST API.
///
/// ## Methods
///
/// | `method`          | Purpose                                                        |
/// |-------------------|----------------------------------------------------------------|
/// | `initialize`      | Legacy handshake. Echoes a supported protocol version.         |
/// | `server/discover` | Stateless capability discovery (modern, handshake-free).       |
/// | `ping`            | Liveness. Returns `{}`.                                         |
/// | `tools/list`      | List the available tools and their input schemas.              |
/// | `tools/call`      | Invoke a tool by name with `arguments`.                        |
///
/// ## Tools
///
/// Discover these (and their argument schemas) at runtime with `tools/list`:
/// `list_documents`, `get_summary`, `search_elements`, `get_element`,
/// `get_element_relationships`, `validate_poam`, and
/// `get_non_eligible_requirements`. Start with `list_documents` to learn the
/// valid `document`/`revision` arguments.
///
/// Tool results come back as **TOON**-encoded text (a token-efficient encoding;
/// the consumer of a tool result is always a model).
///
/// ## Examples
///
/// List the tools:
///
/// ```bash
/// curl -X POST https://api.tolerance.app/mcp \
///   -H 'Content-Type: application/json' \
///   -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
/// ```
///
/// Call a tool:
///
/// ```bash
/// curl -X POST https://api.tolerance.app/mcp \
///   -H 'Content-Type: application/json' \
///   -d '{"jsonrpc":"2.0","id":2,"method":"tools/call",
///        "arguments":{},"params":{"name":"list_documents","arguments":{}}}'
/// ```
///
/// A request with no `id` is a *notification*: it returns `202 Accepted` with
/// an empty body.
#[utoipa::path(
    post,
    path = "/mcp",
    tag = "MCP",
    request_body(
        content = JsonRpcRequest,
        description = "A JSON-RPC 2.0 request. Omit `id` to send a notification.",
    ),
    responses(
        (status = 200, description = "JSON-RPC response — `result` on success, `error` on failure.", body = JsonRpcResponse),
        (status = 202, description = "Notification accepted (the request had no `id`). Empty body."),
        (status = 400, description = "Malformed JSON-RPC, or an `Mcp-Method`/`Mcp-Name` header that disagrees with the body (SEP-2243).", body = JsonRpcResponse),
    ),
)]
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
