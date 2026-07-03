//! `get_summary` — document metadata and element counts by type.

use serde_json::{Value, json};

use crate::cmmc::state::CmmcState;
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::{doc_rev_properties, encode, require_document};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_summary",
        description: "Get a document's metadata and element counts by type (families, requirements, assessment objectives, controls, …). Useful for orienting before searching.",
        input_schema: json!({
            "type": "object",
            "properties": doc_rev_properties(),
            "required": ["document", "revision"],
        }),
    }
}

pub fn call(state: &CmmcState, args: &Value) -> CallToolResult {
    let key = match require_document(state, args) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let doc = state
        .get_document(key)
        .expect("checked by require_document");

    let mut counts: Vec<(String, usize)> = doc
        .index
        .type_counts
        .iter()
        .map(|(t, c)| (format!("{t:?}"), *c))
        .collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let summary = json!({
        "document": key.display_name(),
        "metadata": doc.documents.first(),
        "element_counts": counts.into_iter().map(|(t, c)| json!({ "type": t, "count": c })).collect::<Vec<_>>(),
        "relationship_count": doc.relationships.len(),
    });
    encode(&summary)
}
