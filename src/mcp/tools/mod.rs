//! Tool registry. Each tool lives in its own file and exposes exactly two
//! functions:
//!
//! ```text
//! pub fn definition() -> ToolDefinition
//! pub fn call(state: &CmmcState, args: &Value) -> CallToolResult
//! ```
//!
//! To add a tool: create the file, then add one line to [`REGISTRY`].
//! Shared argument/encoding helpers live in [`support`].

use serde_json::Value;

use crate::cmmc::state::CmmcState;

use super::protocol::{CallToolResult, ToolDefinition};

pub mod support;

pub mod get_element;
pub mod get_element_relationships;
pub mod get_non_eligible_requirements;
pub mod get_summary;
pub mod list_documents;
pub mod search_elements;
pub mod validate_poam;

type DefinitionFn = fn() -> ToolDefinition;
type CallFn = fn(&CmmcState, &Value) -> CallToolResult;

/// Every registered tool: `(name, definition, call)`. The single place to
/// touch when adding a tool.
const REGISTRY: &[(&str, DefinitionFn, CallFn)] = &[
    (
        "list_documents",
        list_documents::definition,
        list_documents::call,
    ),
    ("get_summary", get_summary::definition, get_summary::call),
    (
        "search_elements",
        search_elements::definition,
        search_elements::call,
    ),
    ("get_element", get_element::definition, get_element::call),
    (
        "get_element_relationships",
        get_element_relationships::definition,
        get_element_relationships::call,
    ),
    (
        "validate_poam",
        validate_poam::definition,
        validate_poam::call,
    ),
    (
        "get_non_eligible_requirements",
        get_non_eligible_requirements::definition,
        get_non_eligible_requirements::call,
    ),
];

/// All tool definitions, surfaced via `tools/list`.
pub fn definitions() -> Vec<ToolDefinition> {
    REGISTRY.iter().map(|(_, def, _)| def()).collect()
}

/// Dispatch a `tools/call` by name. Unknown names and bad arguments come back
/// as `isError` tool results (per spec) rather than protocol errors.
pub fn call(state: &CmmcState, name: &str, args: &Value) -> CallToolResult {
    match REGISTRY.iter().find(|(n, _, _)| *n == name) {
        Some((_, _, call)) => call(state, args),
        None => CallToolResult::err(format!("Unknown tool: '{name}'")),
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn registry_names_match_definitions() {
        for (name, def, _) in REGISTRY {
            assert_eq!(
                *name,
                def().name,
                "registry name must match definition name"
            );
        }
    }
}
