//! `get_element` — one element by exact identifier, with its
//! relationship-linked statement/discussion/assessment text resolved.

use serde::Serialize;
use serde_json::{Value, json};

use crate::cmmc::state::CmmcState;
use crate::mcp::constants::{DOC_PARAM_DESC, RELATED_ELEMENTS_LIMIT, REV_PARAM_DESC};
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::{ElementView, encode, require_document, str_arg};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_element",
        description: "Fetch one element by its exact identifier — a requirement (`03.01.01`), an assessment objective (`03.01.01.a`), a control (`AC-2`), a family (`03.01`), etc. Returns the full official text.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "document":   { "type": "string", "description": DOC_PARAM_DESC },
                "revision":   { "type": "string", "description": REV_PARAM_DESC },
                "identifier": { "type": "string", "description": "Exact element identifier, e.g. `03.05.03` or `AC-2`" },
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

    let Some(element) = doc
        .index
        .get_by_identifier(identifier)
        .and_then(|idx| doc.elements.get(idx))
    else {
        return CallToolResult::err(format!(
            "Element '{identifier}' not found in {key}. Try search_elements to locate the right identifier."
        ));
    };

    // In the CPRT data a requirement row is often just a title — the
    // statement, discussion, and assessment-method text live in elements
    // linked by relationships (e.g. `SR-03.05.03`, `D-03.05.03`,
    // `E-/I-/T-03.05.03`). Resolve those so one call returns the full
    // official text.
    #[derive(Serialize)]
    struct RelatedView<'a> {
        relationship: &'a str,
        #[serde(flatten)]
        element: ElementView<'a>,
    }

    let related: Vec<RelatedView> = doc
        .relationships
        .iter()
        .filter(|r| {
            r.source_element_identifier == identifier || r.dest_element_identifier == identifier
        })
        .filter_map(|r| {
            let other = if r.source_element_identifier == identifier {
                &r.dest_element_identifier
            } else {
                &r.source_element_identifier
            };
            doc.index
                .get_by_identifier(other)
                .and_then(|idx| doc.elements.get(idx))
                .map(|e| RelatedView {
                    relationship: &r.relationship_identifier,
                    element: ElementView::from(e),
                })
        })
        .take(RELATED_ELEMENTS_LIMIT)
        .collect();

    let body = json!({
        "element": serde_json::to_value(ElementView::from(element)).unwrap_or_default(),
        "related": serde_json::to_value(&related).unwrap_or_default(),
    });
    encode(&body)
}
