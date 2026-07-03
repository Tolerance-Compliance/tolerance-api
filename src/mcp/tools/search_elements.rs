//! `search_elements` — full-text search over a document's elements.

use serde_json::{Value, json};

use crate::cmmc::handler::query::ElementQuery;
use crate::cmmc::state::CmmcState;
use crate::mcp::constants::{
    DOC_PARAM_DESC, REV_PARAM_DESC, SEARCH_DEFAULT_LIMIT, SEARCH_MAX_LIMIT,
};
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::{ElementView, encode, require_document, str_arg};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "search_elements",
        description: "Full-text search over a document's elements (identifier, title, and text), optionally filtered by element type. Returns matching elements with their full text, TOON-encoded.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "document": { "type": "string", "description": DOC_PARAM_DESC },
                "revision": { "type": "string", "description": REV_PARAM_DESC },
                "query":    { "type": "string", "description": "Search term, e.g. `multifactor authentication`" },
                "type":     { "type": "string", "description": "Optional element type filter: `family`, `requirement`, `security_requirement`, `discussion`, `control`, `control_enhancement`, or assessment-guide types `determination`, `examine`, `interview`, `test`, `odp`, …" },
                "limit":    { "type": "integer", "description": "Max results (default 20, max 200)" },
            },
            "required": ["document", "revision", "query"],
        }),
    }
}

pub fn call(state: &CmmcState, args: &Value) -> CallToolResult {
    let key = match require_document(state, args) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let query = match str_arg(args, "query") {
        Ok(q) => q,
        Err(e) => return e,
    };
    let doc = state
        .get_document(key)
        .expect("checked by require_document");

    // Reuse the REST layer's type-name parsing.
    let type_filter = ElementQuery {
        element_type: args.get("type").and_then(Value::as_str).map(String::from),
        search: None,
        limit: None,
        offset: None,
    }
    .parse_element_type();

    if args.get("type").and_then(Value::as_str).is_some() && type_filter.is_none() {
        return CallToolResult::err(format!(
            "Unknown element type '{}'. See the tool description for valid types.",
            args["type"].as_str().unwrap_or_default()
        ));
    }

    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map(|l| l as usize)
        .unwrap_or(SEARCH_DEFAULT_LIMIT)
        .min(SEARCH_MAX_LIMIT);

    let mut indices = doc.index.search(query, type_filter);
    // The index returns set-ordered indices; sort for deterministic output.
    indices.sort_unstable();
    let total = indices.len();

    let results: Vec<ElementView> = indices
        .into_iter()
        .take(limit)
        .filter_map(|idx| doc.elements.get(idx))
        .map(ElementView::from)
        .collect();

    let body = json!({
        "query": query,
        "total_matches": total,
        "returned": results.len(),
        "results": serde_json::to_value(&results).unwrap_or_default(),
    });
    encode(&body)
}
