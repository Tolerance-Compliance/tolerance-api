//! `get_element_relationships` — every relationship an element participates
//! in (e.g. 171 requirement ↔ 171A assessment objectives, 800-53 sources).

use serde_json::{Value, json};

use crate::cmmc::model::Relationship;
use crate::cmmc::state::CmmcState;
use crate::mcp::constants::{DOC_PARAM_DESC, REV_PARAM_DESC};
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::{encode, require_document, str_arg};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_element_relationships",
        description: "List every relationship an element participates in — e.g. mapping an SP 800-171 requirement to its 800-171A assessment objectives or its 800-53 source controls.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "document":   { "type": "string", "description": DOC_PARAM_DESC },
                "revision":   { "type": "string", "description": REV_PARAM_DESC },
                "identifier": { "type": "string", "description": "Exact element identifier, e.g. `03.05.03`" },
            },
            "required": ["document", "revision", "identifier"],
        }),
    }
}

pub fn call(state: &CmmcState, args: &Value) -> CallToolResult {
    let key = match require_document(state, args) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let identifier = match str_arg(args, "identifier") {
        Ok(i) => i,
        Err(e) => return e,
    };
    let doc = state
        .get_document(key)
        .expect("checked by require_document");

    let related: Vec<&Relationship> = doc
        .relationships
        .iter()
        .filter(|r| {
            r.source_element_identifier == identifier || r.dest_element_identifier == identifier
        })
        .collect();

    let body = json!({
        "identifier": identifier,
        "relationship_count": related.len(),
        "relationships": serde_json::to_value(&related).unwrap_or_default(),
    });
    encode(&body)
}
