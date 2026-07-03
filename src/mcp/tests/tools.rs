//! Tool dispatch tests over real 800-171 r3 catalog data.

use serde_json::json;

use crate::mcp::tools;

use super::fixtures::{call, test_state, text};

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
fn search_elements_accepts_assessment_guide_types() {
    let state = test_state();
    let result = call(
        &state,
        "search_elements",
        json!({
            "document": "sp800-171",
            "revision": "r3",
            "query": "multi-factor",
            "type": "test",
            "limit": 5
        }),
    );
    assert!(!result.is_error, "unexpected error: {}", text(&result));
    assert!(
        text(&result).contains("T-03.05.03"),
        "expected assessment test element in: {}",
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
