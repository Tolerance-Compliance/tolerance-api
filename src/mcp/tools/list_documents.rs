//! `list_documents` — every loaded document/revision pair.

use serde::Serialize;
use serde_json::{Value, json};

use crate::cmmc::state::CmmcState;
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::encode;

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_documents",
        description: "List every loaded standards document and revision (NIST SP 800-53/171/171A/172 and FAR 52.204-21) with its display name. Call this first to discover valid `document`/`revision` arguments for the other tools.",
        input_schema: json!({ "type": "object", "properties": {} }),
    }
}

pub fn call(state: &CmmcState, _args: &Value) -> CallToolResult {
    #[derive(Serialize)]
    struct Doc {
        id: String,
        name: String,
        document: String,
        revision: String,
        source: String,
    }

    let mut docs: Vec<Doc> = state
        .available_documents()
        .into_iter()
        .map(|key| Doc {
            id: key.to_string(),
            name: key.display_name(),
            document: key.document_string(),
            revision: key.revision_string(),
            source: format!("{:?}", key.source()).to_lowercase(),
        })
        .collect();
    docs.sort_by(|a, b| a.id.cmp(&b.id));

    encode(&docs)
}
