//! New NIST API handlers using /api/v1/nist/:document/:revision structure
//!
//! These handlers replace the legacy CMMC level-based API with a more flexible
//! document+revision structure that supports multiple revisions.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};

use crate::cmmc::model::{Document, Element, ElementType, Relationship};
use crate::cmmc::response::{DataSummary, Family, PaginatedResponse, Requirement, SecurityRequirement};
use crate::cmmc::state::CmmcState;
use crate::cmmc::format_response::FormatResponse;
use crate::handler::error::ApiError;

use super::query::{parse_document_key, ElementQuery};

/// Document info for discovery endpoint
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DocumentInfo {
    /// URL path identifier (e.g., "sp800-171/r3")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// NIST document identifier
    pub document: String,
    /// Revision string
    pub revision: String,
}

/// Get list of available documents
#[utoipa::path(
    get,
    path = "/v1/nist/documents",
    responses(
        (status = 200, description = "List of available documents", body = Vec<DocumentInfo>)
    ),
    tag = "NIST"
)]
pub async fn get_documents(
    State(state): State<CmmcState>,
) -> Json<Vec<DocumentInfo>> {
    let mut docs: Vec<DocumentInfo> = state
        .available_documents()
        .into_iter()
        .map(|key| DocumentInfo {
            id: key.to_string(),
            name: key.display_name(),
            document: key.document.to_string(),
            revision: key.revision.to_string(),
        })
        .collect();

    docs.sort_by(|a, b| a.id.cmp(&b.id));
    Json(docs)
}

/// Get summary for a specific document+revision
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/summary",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)")
    ),
    responses(
        (status = 200, description = "Summary retrieved successfully", body = DataSummary),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_summary(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
) -> Result<Json<DataSummary>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let data = state.data(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let index = state.index(key).unwrap();

    let summary = DataSummary {
        document: data.response.elements.documents.first().cloned().unwrap_or_else(|| {
            Document {
                doc_identifier: String::new(),
                name: String::new(),
                version: String::new(),
                website: String::new(),
            }
        }),
        family_count: index.count_by_type(ElementType::Family),
        requirement_count: index.count_by_type(ElementType::Requirement),
        security_requirement_count: index.count_by_type(ElementType::SecurityRequirement),
        relationship_count: data.response.elements.relationships.len(),
    };

    Ok(Json(summary))
}

/// Get all families
///
/// Returns a list of all NIST control families with their requirements.
///
/// # Content Negotiation
///
/// This endpoint supports content negotiation via the `Accept` header:
///
/// - `Accept: application/json` (default): Standard JSON response
/// - `Accept: text/toon`: Token-Oriented Object Notation - a compact, LLM-friendly format
///   that reduces token usage by 30-40%. Recommended when passing data to LLMs for
///   compliance guidance.
///
/// ## TOON Format Example
///
/// ```text
/// [2]{identifier,title,requirements}:
///   AC,"Access Control",[3]{identifier,title,text}:...
///   AU,"Audit and Accountability",[5]{identifier,title,text}:...
/// ```
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/families",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)")
    ),
    responses(
        (status = 200, description = "Success. Returns application/json by default, or text/toon if requested via Accept header", body = Vec<Family>),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_families(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Family>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let relationships = &state.data(key).unwrap().response.elements.relationships;
    let family_indices = state.index(key).unwrap().get_by_type(ElementType::Family);

    let families: Vec<Family> = family_indices
        .iter()
        .filter_map(|&idx| elements.get(idx))
        .map(|family: &Element| build_family(family, elements, relationships))
        .collect();

    // Check Accept header for TOON format
    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(families, use_toon))
}

/// Get a specific family by identifier
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/families/{id}",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ("id" = String, Path, description = "Family identifier"),
        ("format" = Option<String>, Query, description = "Output format: json (default) or toon")
    ),
    responses(
        (status = 200, description = "Family details", body = Family),
        (status = 404, description = "Family not found")
    ),
    tag = "NIST"
)]
pub async fn get_family(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Family>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let relationships = &state.data(key).unwrap().response.elements.relationships;
    let index = state.index(key).unwrap();

    let idx = index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    let family = elements
        .get(idx)
        .filter(|e: &&Element| e.element_type == ElementType::Family)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(build_family(family, elements, relationships), use_toon))
}

/// Get all elements with optional filtering and pagination
///
/// # Content Negotiation
///
/// Use `Accept: text/toon` for LLM-optimized output.
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/elements",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ElementQuery
    ),
    responses(
        (status = 200, description = "Paginated list of elements. Supports Accept: application/json (default) or Accept: text/toon", body = PaginatedResponse<Element>),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_elements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    Query(query): Query<ElementQuery>,
    headers: HeaderMap,
) -> Result<FormatResponse<PaginatedResponse<Element>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let index = state.index(key).unwrap();

    let filtered_indices: Vec<usize> = match (query.parse_element_type(), &query.search) {
        (Some(element_type), Some(search_term)) => {
            index.search(search_term, Some(element_type))
        }
        (Some(element_type), None) => index.get_by_type(element_type).to_vec(),
        (None, Some(search_term)) => index.search(search_term, None),
        (None, None) => (0..elements.len()).collect(),
    };

    let total = filtered_indices.len();
    let offset = query.offset();
    let limit = query.limit();

    let items: Vec<Element> = filtered_indices
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|idx| elements.get(idx))
        .cloned()
        .collect();

    let has_more = offset + limit < total;

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(
        PaginatedResponse {
            data: items,
            total,
            offset,
            limit,
            has_more,
        },
        use_toon,
    ))
}

/// Get a specific element by identifier
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/elements/{id}",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element details", body = Element),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
pub async fn get_element(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Element>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let index = state.index(key).unwrap();

    let idx = index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let element = elements
        .get(idx)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(element, use_toon))
}

/// Get all requirements
///
/// Returns all requirements across all families. Use the `format` query parameter
/// to get TOON format for LLM consumption (30-40% token reduction).
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/requirements",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ("format" = Option<String>, Query, description = "Output format: 'json' (default) or 'toon'. TOON optimized for LLM consumption.")
    ),
    responses(
        (status = 200, description = "List of requirements", body = Vec<Requirement>),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Requirement>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let relationships = &state.data(key).unwrap().response.elements.relationships;
    let requirement_indices = state.index(key).unwrap().get_by_type(ElementType::Requirement);

    let requirements: Vec<Requirement> = requirement_indices
        .iter()
        .filter_map(|&idx| elements.get(idx))
        .map(|req: &Element| {
            let security_requirements = get_security_requirements_for_requirement(req, elements, relationships);
            Requirement {
                identifier: req.element_identifier.clone(),
                title: req.title.clone(),
                text: req.text.clone(),
                security_requirements,
            }
        })
        .collect();

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(requirements, use_toon))
}

/// Get all security requirements
///
/// Returns all security requirements with discussion and assessment text.
/// Use TOON format (`?format=toon`) for LLM-optimized output.
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/security-requirements",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ("format" = Option<String>, Query, description = "Output format: 'json' (default) or 'toon'. TOON optimized for LLM consumption.")
    ),
    responses(
        (status = 200, description = "List of security requirements", body = Vec<SecurityRequirement>),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_security_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<SecurityRequirement>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let sr_indices = state.index(key).unwrap().get_by_type(ElementType::SecurityRequirement);

    let security_requirements: Vec<SecurityRequirement> = sr_indices
        .iter()
        .filter_map(|&idx| elements.get(idx))
        .map(|sr: &Element| {
            let discussion = find_related_text(elements, &sr.element_identifier, ElementType::Discussion);
            let assessment = find_related_text(elements, &sr.element_identifier, ElementType::Assessment);

            SecurityRequirement {
                identifier: sr.element_identifier.clone(),
                title: sr.title.clone(),
                text: sr.text.clone(),
                discussion,
                assessment,
            }
        })
        .collect();

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(security_requirements, use_toon))
}

/// Get all relationships
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/relationships",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)")
    ),
    responses(
        (status = 200, description = "List of relationships", body = Vec<Relationship>),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_relationships(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let data = state.data(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(data.response.elements.relationships.clone(), use_toon))
}

/// Get relationships for a specific element
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/elements/{id}/relationships",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element relationships", body = Vec<Relationship>),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    let index = state.index(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    // Verify element exists
    index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let data = state.data(key).unwrap();
    let relationships: Vec<Relationship> = data
        .response
        .elements
        .relationships
        .iter()
        .filter(|r| r.source_element_identifier == id || r.dest_element_identifier == id)
        .cloned()
        .collect();

    let use_toon = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/toon"))
        .unwrap_or(false);

    Ok(FormatResponse::with_format(relationships, use_toon))
}

// Helper functions

fn build_family(family: &Element, elements: &[Element], relationships: &[Relationship]) -> Family {
    let requirements = get_family_requirements(family, elements, relationships);
    Family {
        identifier: family.element_identifier.clone(),
        title: family.title.clone(),
        requirements,
    }
}

fn get_family_requirements(
    family: &Element,
    elements: &[Element],
    relationships: &[Relationship],
) -> Vec<Requirement> {
    let family_prefix = &family.element_identifier;

    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::Requirement
                && e.element_identifier.starts_with(family_prefix)
                && e.element_identifier.len() > family_prefix.len()
        })
        .map(|req| {
            let security_requirements = get_security_requirements_for_requirement(req, elements, relationships);
            Requirement {
                identifier: req.element_identifier.clone(),
                title: req.title.clone(),
                text: req.text.clone(),
                security_requirements,
            }
        })
        .collect()
}

fn get_security_requirements_for_requirement(
    requirement: &Element,
    elements: &[Element],
    _relationships: &[Relationship],
) -> Vec<SecurityRequirement> {
    let req_id = &requirement.element_identifier;
    let sr_prefix = format!("SR-{}", req_id);

    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::SecurityRequirement
                && e.element_identifier.starts_with(&sr_prefix)
        })
        .map(|sr| {
            let discussion = find_related_text(elements, &sr.element_identifier, ElementType::Discussion);
            let assessment = find_related_text(elements, &sr.element_identifier, ElementType::Assessment);

            SecurityRequirement {
                identifier: sr.element_identifier.clone(),
                title: sr.title.clone(),
                text: sr.text.clone(),
                discussion,
                assessment,
            }
        })
        .collect()
}

fn find_related_text(
    elements: &[Element],
    sr_identifier: &str,
    element_type: ElementType,
) -> Option<String> {
    elements
        .iter()
        .find(|e| e.element_type == element_type && e.element_identifier.contains(sr_identifier))
        .map(|e| e.text.clone())
        .filter(|t| !t.is_empty())
}
