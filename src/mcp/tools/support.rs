//! Helpers shared by all tools: argument parsing, document resolution, and
//! TOON encoding of results.

use serde::Serialize;
use serde_json::{Value, json};

use crate::cmmc::handler::query::{parse_far_document_key, parse_nist_document_key};
use crate::cmmc::model::{DocumentKey, Element};
use crate::cmmc::state::CmmcState;

use crate::mcp::constants::{DOC_PARAM_DESC, REV_PARAM_DESC};
use crate::mcp::protocol::CallToolResult;

/// JSON-schema properties for the ubiquitous `document`/`revision` pair.
pub fn doc_rev_properties() -> Value {
    json!({
        "document": { "type": "string", "description": DOC_PARAM_DESC },
        "revision": { "type": "string", "description": REV_PARAM_DESC },
    })
}

/// Encode a value as TOON text (LLM-optimized); fall back to JSON if the
/// shape isn't TOON-encodable.
pub fn encode<T: Serialize>(value: &T) -> CallToolResult {
    match toon_format::encode(value, &toon_format::EncodeOptions::new()) {
        Ok(toon) => CallToolResult::ok(toon),
        Err(_) => match serde_json::to_string_pretty(value) {
            Ok(json) => CallToolResult::ok(json),
            Err(e) => CallToolResult::err(format!("Serialization error: {e}")),
        },
    }
}

/// Extract a required string argument.
pub fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, CallToolResult> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| CallToolResult::err(format!("Missing required string argument '{key}'")))
}

/// Parse `document`/`revision` arguments into a key, accepting both NIST and
/// FAR document names (the REST API splits these across two path prefixes).
pub fn parse_key(args: &Value) -> Result<DocumentKey, CallToolResult> {
    let document = str_arg(args, "document")?;
    let revision = str_arg(args, "revision")?;
    parse_nist_document_key(document, revision)
        .or_else(|_| parse_far_document_key(document, revision))
        .map_err(|e| CallToolResult::err(e.to_string()))
}

/// Parse the key and confirm the document is actually loaded.
pub fn require_document(state: &CmmcState, args: &Value) -> Result<DocumentKey, CallToolResult> {
    let key = parse_key(args)?;
    if state.get_document(key).is_none() {
        return Err(CallToolResult::err(format!(
            "Document {key} not loaded. Call list_documents for available documents."
        )));
    }
    Ok(key)
}

/// Borrowed, serialization-friendly view of an [`Element`].
#[derive(Serialize)]
pub struct ElementView<'a> {
    pub identifier: &'a str,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: &'a str,
    pub text: &'a str,
}

impl<'a> From<&'a Element> for ElementView<'a> {
    fn from(e: &'a Element) -> Self {
        Self {
            identifier: &e.element_identifier,
            kind: format!("{:?}", e.element_type),
            title: &e.title,
            text: &e.text,
        }
    }
}
