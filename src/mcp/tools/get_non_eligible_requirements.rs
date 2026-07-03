//! `get_non_eligible_requirements` — requirements that can never be deferred
//! to a POA&M under CMMC scoring.

use serde_json::{Value, json};

use crate::cmmc::state::CmmcState;
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::{doc_rev_properties, encode, require_document};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_non_eligible_requirements",
        description: "List every SP 800-171 requirement that can NEVER be placed on a POA&M under the CMMC scoring methodology (must be fully implemented at assessment time).",
        input_schema: json!({
            "type": "object",
            "properties": doc_rev_properties(),
            "required": ["document", "revision"],
        }),
    }
}

pub fn call(state: &CmmcState, args: &Value) -> CallToolResult {
    if let Err(e) = require_document(state, args) {
        return e;
    }
    let non_eligible = state.poam_validator().get_non_eligible_requirements();
    let body = json!({
        "count": non_eligible.len(),
        "requirement_ids": non_eligible,
        "note": "These requirements must be fully implemented at assessment time; they can never be deferred to a POA&M under CMMC scoring.",
    });
    encode(&body)
}
