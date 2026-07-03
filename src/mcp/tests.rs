//! Tests for the MCP surface: tool dispatch over real catalog data and
//! handler-level protocol behavior (legacy handshake, modern statelessness,
//! SEP-2243 header validation).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::cmmc::model::{DocumentKey, DocumentRevision, NistDocument};
use crate::cmmc::state::CmmcState;

use super::tools;

fn test_state() -> CmmcState {
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

fn call(state: &CmmcState, name: &str, args: Value) -> super::protocol::CallToolResult {
    tools::call(state, name, &args)
}

fn text(result: &super::protocol::CallToolResult) -> &str {
    &result.content[0].text
}

// ---------------------------------------------------------------------------
// Tool dispatch
// ---------------------------------------------------------------------------

#[test]
fn list_documents_returns_loaded_docs() {
    let state = test_state();
    let result = call(&state, "list_documents", json!({}));
    assert!(!result.is_error);
    assert!(text(&result).contains("sp800-171/r3"));
    assert!(text(&result).contains("SP 800-171 Rev 3"));
}

#[test]
fn get_summary_counts_elements() {
    let state = test_state();
    let result = call(
        &state,
        "get_summary",
        json!({ "document": "sp800-171", "revision": "r3" }),
    );
    assert!(!result.is_error);
    assert!(text(&result).contains("element_counts"));
    assert!(text(&result).contains("relationship_count"));
}

#[test]
fn search_elements_finds_mfa() {
    let state = test_state();
    let result = call(
        &state,
        "search_elements",
        json!({
            "document": "sp800-171",
            "revision": "r3",
            "query": "multi-factor authentication",
            "type": "requirement",
            "limit": 5
        }),
    );
    assert!(!result.is_error);
    assert!(
        text(&result).contains("03.05.03"),
        "expected MFA requirement in: {}",
        text(&result)
    );
}

#[test]
fn search_elements_rejects_unknown_type() {
    let state = test_state();
    let result = call(
        &state,
        "search_elements",
        json!({
            "document": "sp800-171",
            "revision": "r3",
            "query": "access",
            "type": "banana"
        }),
    );
    assert!(result.is_error);
    assert!(text(&result).contains("Unknown element type"));
}

#[test]
fn get_element_by_identifier() {
    let state = test_state();
    let result = call(
        &state,
        "get_element",
        json!({ "document": "sp800-171", "revision": "r3", "identifier": "03.05.03" }),
    );
    assert!(!result.is_error);
    assert!(text(&result).contains("03.05.03"));
    // The linked security_requirement carries the actual statement text.
    assert!(
        text(&result).contains("SR-03.05.03"),
        "expected related statement elements in: {}",
        text(&result)
    );
}

#[test]
fn get_element_unknown_identifier_is_tool_error() {
    let state = test_state();
    let result = call(
        &state,
        "get_element",
        json!({ "document": "sp800-171", "revision": "r3", "identifier": "99.99.99" }),
    );
    assert!(result.is_error);
    assert!(text(&result).contains("not found"));
}

#[test]
fn get_element_relationships_lists_edges() {
    let state = test_state();
    let result = call(
        &state,
        "get_element_relationships",
        json!({ "document": "sp800-171", "revision": "r3", "identifier": "03.05.03" }),
    );
    assert!(!result.is_error);
    assert!(text(&result).contains("relationship_count"));
}

#[test]
fn validate_poam_batch() {
    let state = test_state();
    let result = call(
        &state,
        "validate_poam",
        json!({
            "document": "sp800-171",
            "revision": "r3",
            "requirement_ids": ["03.01.01", "03.05.03"]
        }),
    );
    assert!(!result.is_error);
    assert!(text(&result).contains("eligible_count"));
}

#[test]
fn validate_poam_requires_ids() {
    let state = test_state();
    let result = call(
        &state,
        "validate_poam",
        json!({ "document": "sp800-171", "revision": "r3", "requirement_ids": [] }),
    );
    assert!(result.is_error);
}

#[test]
fn non_eligible_requirements_listed() {
    let state = test_state();
    let result = call(
        &state,
        "get_non_eligible_requirements",
        json!({ "document": "sp800-171", "revision": "r3" }),
    );
    assert!(!result.is_error);
    assert!(text(&result).contains("requirement_ids"));
}

#[test]
fn unknown_tool_is_tool_error() {
    let state = test_state();
    let result = call(&state, "does_not_exist", json!({}));
    assert!(result.is_error);
    assert!(text(&result).contains("Unknown tool"));
}

#[test]
fn unloaded_document_is_tool_error() {
    let state = test_state();
    let result = call(
        &state,
        "get_summary",
        json!({ "document": "sp800-172", "revision": "v1" }),
    );
    assert!(result.is_error);
    assert!(text(&result).contains("not loaded"));
}

#[test]
fn definitions_are_well_formed() {
    let defs = tools::definitions();
    assert_eq!(defs.len(), 7);
    for def in &defs {
        assert!(!def.name.is_empty());
        assert!(!def.description.is_empty());
        assert_eq!(def.input_schema["type"], "object");
    }
}

// ---------------------------------------------------------------------------
// Handler / protocol behavior (through the real router)
// ---------------------------------------------------------------------------

async fn post_mcp(body: Value, headers: &[(&str, &str)]) -> (StatusCode, Option<Value>) {
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

#[tokio::test]
async fn legacy_initialize_negotiates_version() {
    let (status, body) = post_mcp(
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0.0.0" }
            }
        }),
        &[],
    )
    .await;
    let body = body.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(body["result"]["serverInfo"]["name"], "tolerance-api");
}

#[tokio::test]
async fn server_discover_is_stateless_capability_discovery() {
    let (status, body) = post_mcp(
        json!({ "jsonrpc": "2.0", "id": 1, "method": "server/discover" }),
        &[],
    )
    .await;
    let body = body.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["protocolVersion"], "2026-07-28");
}

#[tokio::test]
async fn tools_call_works_without_prior_initialize() {
    // Stateless: a tools/call with no handshake must succeed.
    let (status, body) = post_mcp(
        json!({
            "jsonrpc": "2.0", "id": 7, "method": "tools/call",
            "params": {
                "name": "list_documents",
                "arguments": {},
                "_meta": { "protocolVersion": "2026-07-28" }
            }
        }),
        &[
            ("mcp-protocol-version", "2026-07-28"),
            ("mcp-method", "tools/call"),
            ("mcp-name", "list_documents"),
        ],
    )
    .await;
    let body = body.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["content"][0]["type"], "text");
}

#[tokio::test]
async fn tools_list_returns_all_tools() {
    let (status, body) = post_mcp(
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        &[],
    )
    .await;
    let body = body.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["tools"].as_array().unwrap().len(), 7);
}

#[tokio::test]
async fn mismatched_mcp_method_header_rejected() {
    let (status, body) = post_mcp(
        json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/list" }),
        &[("mcp-method", "tools/call")],
    )
    .await;
    let body = body.unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("HeaderMismatch")
    );
}

#[tokio::test]
async fn mismatched_mcp_name_header_rejected() {
    let (status, _) = post_mcp(
        json!({
            "jsonrpc": "2.0", "id": 4, "method": "tools/call",
            "params": { "name": "list_documents", "arguments": {} }
        }),
        &[("mcp-name", "get_element")],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn notification_gets_202_no_body() {
    let (status, body) = post_mcp(
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(body.is_none());
}

#[tokio::test]
async fn unknown_method_is_method_not_found() {
    let (status, body) = post_mcp(
        json!({ "jsonrpc": "2.0", "id": 5, "method": "resources/list" }),
        &[],
    )
    .await;
    let body = body.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["error"]["code"], -32601);
}

#[tokio::test]
async fn malformed_json_is_400() {
    let app = crate::routing::app(test_state());
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
