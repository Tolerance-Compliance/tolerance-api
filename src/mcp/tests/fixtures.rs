//! Shared test fixtures: real 800-171 r3 catalog state and request helpers.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

use crate::cmmc::model::{DocumentKey, DocumentRevision, NistDocument};
use crate::cmmc::state::CmmcState;
use crate::mcp::protocol::CallToolResult;
use crate::mcp::tools;

/// State loaded from the real SP 800-171 r3 data file.
pub fn test_state() -> CmmcState {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/cprt-sp_800_171_3_0_0-20260215-171034.json"
    );
    let data = CmmcState::load_json(path).expect("load 800-171 r3 fixture");
    CmmcState::new(vec![(
        DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev3),
        data,
    )])
}

/// Dispatch a tool call directly (no HTTP).
pub fn call(state: &CmmcState, name: &str, args: Value) -> CallToolResult {
    tools::call(state, name, &args)
}

/// The text of a tool result's first content block.
pub fn text(result: &CallToolResult) -> &str {
    &result.content[0].text
}

/// POST a JSON-RPC body to `/mcp` through the real router, with optional
/// extra headers. Returns the status and the parsed response body (if any).
pub async fn post_mcp(body: Value, headers: &[(&str, &str)]) -> (StatusCode, Option<Value>) {
    let app = crate::routing::app(test_state());
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json");
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    let request = builder.body(Body::from(body.to_string())).unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed = serde_json::from_slice(&bytes).ok();
    (status, parsed)
}

/// POST a raw (possibly malformed) body to `/mcp`.
pub async fn post_mcp_raw(body: &str) -> StatusCode {
    let app = crate::routing::app(test_state());
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    app.oneshot(request).await.unwrap().status()
}
