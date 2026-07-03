//! Protocol behavior through the real router: legacy handshake, modern
//! statelessness, SEP-2243 header validation, notifications, and errors.

use axum::http::StatusCode;
use serde_json::json;

use super::fixtures::{post_mcp, post_mcp_raw};

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
    let status = post_mcp_raw("{not json").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
