//! /v1/far/:document/:revision/*

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
};

use crate::cmmc::model::{Element, Relationship};
use crate::cmmc::response::{DataSummary, Family, PaginatedResponse};
use crate::cmmc::state::CmmcState;
use crate::cmmc::format_response::{FormatResponse, wants_toon};
use crate::handler::error::ApiError;

use super::query::{parse_far_document_key, ElementQuery};
use super::nist::{build_family, build_requirement};

/// Get summary for a specific FAR document+revision.
#[utoipa::path(
    get,
    operation_id = "far_get_summary",
    path = "/v1/far/{document}/{revision}/summary",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`")
    ),
    responses(
        (status = 200, description = "Summary retrieved successfully. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens)."),
        (status = 404, description = "Document not found")
    ),
    tag = "FAR"
)]
pub async fn get_summary(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<DataSummary>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let summary = DataSummary {
        document: doc.documents.first().cloned().unwrap_or_else(|| crate::cmmc::model::Document {
            doc_identifier: String::new(),
            name: String::new(),
            version: String::new(),
            website: String::new(),
        }),
        family_count:                doc.index.count_by_type(crate::cmmc::model::ElementType::Family),
        requirement_count:           doc.index.count_by_type(crate::cmmc::model::ElementType::Requirement),
        security_requirement_count:  doc.index.count_by_type(crate::cmmc::model::ElementType::SecurityRequirement),
        relationship_count:          doc.relationships.len(),
    };

    Ok(FormatResponse::with_format(summary, wants_toon(&headers)))
}

/// Get all families.
#[utoipa::path(
    get,
    operation_id = "far_get_families",
    path = "/v1/far/{document}/{revision}/families",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`")
    ),
    responses(
        (status = 200, description = "List of families."),
        (status = 404, description = "Document not found")
    ),
    tag = "FAR"
)]
pub async fn get_families(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Family>>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let families = doc.index
        .get_by_type(crate::cmmc::model::ElementType::Family)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|family| build_family(family, doc.elements, state.scoring_db(), state.poam_validator()))
        .collect();

    Ok(FormatResponse::with_format(families, wants_toon(&headers)))
}

/// Get a specific family by identifier.
#[utoipa::path(
    get,
    operation_id = "far_get_family",
    path = "/v1/far/{document}/{revision}/families/{id}",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`"),
        ("id" = String, Path, description = "Family identifier")
    ),
    responses(
        (status = 200, description = "Family details."),
        (status = 404, description = "Family not found")
    ),
    tag = "FAR"
)]
pub async fn get_family(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Family>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let idx = doc.index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    let family = doc.elements
        .get(idx)
        .filter(|e| e.element_type == crate::cmmc::model::ElementType::Family)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    Ok(FormatResponse::with_format(
        build_family(family, doc.elements, state.scoring_db(), state.poam_validator()),
        wants_toon(&headers)
    ))
}

/// Get all elements.
#[utoipa::path(
    get,
    operation_id = "far_get_elements",
    path = "/v1/far/{document}/{revision}/elements",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`"),
        ElementQuery
    ),
    responses(
        (status = 200, description = "Paginated list of elements."),
        (status = 404, description = "Document not found")
    ),
    tag = "FAR"
)]
pub async fn get_elements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    Query(query): Query<ElementQuery>,
    headers: HeaderMap,
) -> Result<FormatResponse<PaginatedResponse<Element>>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let filtered: Vec<usize> = match (query.parse_element_type(), &query.search) {
        (Some(et), Some(term)) => doc.index.search(term, Some(et)),
        (Some(et), None)       => doc.index.get_by_type(et).to_vec(),
        (None, Some(term))     => doc.index.search(term, None),
        (None, None)           => (0..doc.elements.len()).collect(),
    };

    let total  = filtered.len();
    let offset = query.offset();
    let limit  = query.limit();

    let items = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|idx| doc.elements.get(idx))
        .cloned()
        .collect();

    Ok(FormatResponse::with_format(
        PaginatedResponse { data: items, total, offset, limit, has_more: offset + limit < total },
        wants_toon(&headers),
    ))
}

/// Get a specific element by identifier.
#[utoipa::path(
    get,
    operation_id = "far_get_element",
    path = "/v1/far/{document}/{revision}/elements/{id}",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element details."),
        (status = 404, description = "Element not found")
    ),
    tag = "FAR"
)]
pub async fn get_element(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Element>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let element = doc.index
        .get_by_identifier(&id)
        .and_then(|idx| doc.elements.get(idx))
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    Ok(FormatResponse::with_format(element, wants_toon(&headers)))
}

/// Get all requirements across all families.
#[utoipa::path(
    get,
    operation_id = "far_get_requirements",
    path = "/v1/far/{document}/{revision}/requirements",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`")
    ),
    responses(
        (status = 200, description = "List of requirements."),
        (status = 404, description = "Document not found")
    ),
    tag = "FAR"
)]
pub async fn get_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<crate::cmmc::response::Requirement>>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let requirements = doc.index
        .get_by_type(crate::cmmc::model::ElementType::Requirement)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|req| build_requirement(req, doc.elements, state.scoring_db(), state.poam_validator()))
        .collect();

    Ok(FormatResponse::with_format(requirements, wants_toon(&headers)))
}

/// Get all relationships.
#[utoipa::path(
    get,
    operation_id = "far_get_relationships",
    path = "/v1/far/{document}/{revision}/relationships",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`")
    ),
    responses(
        (status = 200, description = "List of relationships."),
        (status = 404, description = "Document not found")
    ),
    tag = "FAR"
)]
pub async fn get_relationships(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    Ok(FormatResponse::with_format(doc.relationships.to_vec(), wants_toon(&headers)))
}

/// Get relationships for a specific element.
#[utoipa::path(
    get,
    operation_id = "far_get_element_relationships",
    path = "/v1/far/{document}/{revision}/elements/{id}/relationships",
    params(
        ("document" = String, Path, description = "FAR document: `52.204-21`"),
        ("revision" = String, Path, description = "Document revision: `v2`"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element relationships."),
        (status = 404, description = "Element not found")
    ),
    tag = "FAR"
)]
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_far_document_key(&document, &revision)?;
    let doc = state.get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    doc.index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let relationships = doc.relationships
        .iter()
        .filter(|r| r.source_element_identifier == id || r.dest_element_identifier == id)
        .cloned()
        .collect();

    Ok(FormatResponse::with_format(relationships, wants_toon(&headers)))
}
