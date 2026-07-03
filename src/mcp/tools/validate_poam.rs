//! `validate_poam` — CMMC POA&M eligibility for a batch of requirements.

use serde_json::{Value, json};

use crate::cmmc::poam::PoamEligibility;
use crate::cmmc::state::CmmcState;
use crate::mcp::constants::{DOC_PARAM_DESC, REV_PARAM_DESC};
use crate::mcp::protocol::{CallToolResult, ToolDefinition};

use super::support::{encode, require_document};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "validate_poam",
        description: "Check CMMC POA&M eligibility for one or more SP 800-171 requirement identifiers: whether each may be placed on a Plan of Action & Milestones instead of being fully implemented at assessment time, with the scoring rationale.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "document": { "type": "string", "description": DOC_PARAM_DESC },
                "revision": { "type": "string", "description": REV_PARAM_DESC },
                "requirement_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Requirement identifiers, e.g. [\"03.01.01\", \"03.05.03\"]",
                },
            },
            "required": ["document", "revision", "requirement_ids"],
        }),
    }
}

pub fn call(state: &CmmcState, args: &Value) -> CallToolResult {
    if let Err(e) = require_document(state, args) {
        return e;
    }
    let ids: Vec<String> = match args.get("requirement_ids").and_then(Value::as_array) {
        Some(arr) => arr
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect(),
        None => {
            return CallToolResult::err(
                "Missing required array argument 'requirement_ids'".to_string(),
            );
        }
    };
    if ids.is_empty() {
        return CallToolResult::err("'requirement_ids' must contain at least one identifier");
    }

    let validations = state.poam_validator().validate_batch(&ids);
    let count = |e: PoamEligibility| validations.iter().filter(|v| v.eligibility == e).count();

    let body = json!({
        "total": validations.len(),
        "eligible_count": count(PoamEligibility::Eligible),
        "not_eligible_count": count(PoamEligibility::NotEligible),
        "conditional_count": count(PoamEligibility::Conditional),
        "validations": serde_json::to_value(&validations).unwrap_or_default(),
    });
    encode(&body)
}
