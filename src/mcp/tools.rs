//! Tool definitions and dispatch. Each tool is a thin, read-only view over
//! [`CmmcState`] — the same in-memory indexes the REST handlers use.

use serde::Serialize;
use serde_json::{Value, json};

use crate::cmmc::handler::query::{ElementQuery, parse_far_document_key, parse_nist_document_key};
use crate::cmmc::model::{DocumentKey, Element};
use crate::cmmc::poam::PoamEligibility;
use crate::cmmc::state::CmmcState;

use super::protocol::{CallToolResult, ToolDefinition};

/// Default / maximum result counts for `search_elements`.
const SEARCH_DEFAULT_LIMIT: usize = 20;
const SEARCH_MAX_LIMIT: usize = 200;

const DOC_PARAM_DESC: &str = "Document: `sp800-53`, `sp800-53a`, `sp800-53b`, `sp800-171`, \
     `sp800-171a`, `sp800-172`, `sp800-172a`, or `52.204-21` (FAR)";
const REV_PARAM_DESC: &str =
    "Revision: `r5` (800-53*), `r2`/`r3` (171/171a), `v1` (172/172a), `v2` (FAR)";

fn doc_rev_properties() -> Value {
    json!({
        "document": { "type": "string", "description": DOC_PARAM_DESC },
        "revision": { "type": "string", "description": REV_PARAM_DESC },
    })
}

/// All tools surfaced via `tools/list`.
pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "list_documents",
            description: "List every loaded standards document and revision (NIST SP 800-53/171/171A/172 and FAR 52.204-21) with its display name. Call this first to discover valid `document`/`revision` arguments for the other tools.",
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolDefinition {
            name: "get_summary",
            description: "Get a document's metadata and element counts by type (families, requirements, assessment objectives, controls, …). Useful for orienting before searching.",
            input_schema: json!({
                "type": "object",
                "properties": doc_rev_properties(),
                "required": ["document", "revision"],
            }),
        },
        ToolDefinition {
            name: "search_elements",
            description: "Full-text search over a document's elements (identifier, title, and text), optionally filtered by element type. Returns matching elements with their full text, TOON-encoded.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "document": { "type": "string", "description": DOC_PARAM_DESC },
                    "revision": { "type": "string", "description": REV_PARAM_DESC },
                    "query":    { "type": "string", "description": "Search term, e.g. `multifactor authentication`" },
                    "type":     { "type": "string", "description": "Optional element type filter: `family`, `requirement`, `security_requirement`, `assessment`, `discussion`, `control`, `control_enhancement`, …" },
                    "limit":    { "type": "integer", "description": "Max results (default 20, max 200)" },
                },
                "required": ["document", "revision", "query"],
            }),
        },
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
        },
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
        },
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
        },
        ToolDefinition {
            name: "get_non_eligible_requirements",
            description: "List every SP 800-171 requirement that can NEVER be placed on a POA&M under the CMMC scoring methodology (must be fully implemented at assessment time).",
            input_schema: json!({
                "type": "object",
                "properties": doc_rev_properties(),
                "required": ["document", "revision"],
            }),
        },
    ]
}

/// Dispatch a `tools/call` by name. Unknown names and bad arguments come back
/// as `isError` tool results (per spec) rather than protocol errors.
pub fn call(state: &CmmcState, name: &str, args: &Value) -> CallToolResult {
    match name {
        "list_documents" => list_documents(state),
        "get_summary" => get_summary(state, args),
        "search_elements" => search_elements(state, args),
        "get_element" => get_element(state, args),
        "get_element_relationships" => get_element_relationships(state, args),
        "validate_poam" => validate_poam(state, args),
        "get_non_eligible_requirements" => get_non_eligible_requirements(state, args),
        other => CallToolResult::err(format!("Unknown tool: '{other}'")),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Encode a value as TOON text (LLM-optimized); fall back to JSON if the
/// shape isn't TOON-encodable.
fn encode<T: Serialize>(value: &T) -> CallToolResult {
    match toon_format::encode(value, &toon_format::EncodeOptions::new()) {
        Ok(toon) => CallToolResult::ok(toon),
        Err(_) => match serde_json::to_string_pretty(value) {
            Ok(json) => CallToolResult::ok(json),
            Err(e) => CallToolResult::err(format!("Serialization error: {e}")),
        },
    }
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, CallToolResult> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| CallToolResult::err(format!("Missing required string argument '{key}'")))
}

/// Parse `document`/`revision` arguments into a key, accepting both NIST and
/// FAR document names (the REST API splits these across two path prefixes).
fn parse_key(args: &Value) -> Result<DocumentKey, CallToolResult> {
    let document = str_arg(args, "document")?;
    let revision = str_arg(args, "revision")?;
    parse_nist_document_key(document, revision)
        .or_else(|_| parse_far_document_key(document, revision))
        .map_err(|e| CallToolResult::err(e.to_string()))
}

fn require_document(state: &CmmcState, args: &Value) -> Result<DocumentKey, CallToolResult> {
    let key = parse_key(args)?;
    if state.get_document(key).is_none() {
        return Err(CallToolResult::err(format!(
            "Document {key} not loaded. Call list_documents for available documents."
        )));
    }
    Ok(key)
}

#[derive(Serialize)]
struct ElementView<'a> {
    identifier: &'a str,
    #[serde(rename = "type")]
    kind: String,
    title: &'a str,
    text: &'a str,
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

// ---------------------------------------------------------------------------
// Tools
// ---------------------------------------------------------------------------

fn list_documents(state: &CmmcState) -> CallToolResult {
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

fn get_summary(state: &CmmcState, args: &Value) -> CallToolResult {
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

fn search_elements(state: &CmmcState, args: &Value) -> CallToolResult {
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

fn get_element(state: &CmmcState, args: &Value) -> CallToolResult {
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
        .take(50)
        .collect();

    let body = json!({
        "element": serde_json::to_value(ElementView::from(element)).unwrap_or_default(),
        "related": serde_json::to_value(&related).unwrap_or_default(),
    });
    encode(&body)
}

fn get_element_relationships(state: &CmmcState, args: &Value) -> CallToolResult {
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

    let related: Vec<&crate::cmmc::model::Relationship> = doc
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

fn validate_poam(state: &CmmcState, args: &Value) -> CallToolResult {
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

fn get_non_eligible_requirements(state: &CmmcState, args: &Value) -> CallToolResult {
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
