//! /v1/nist/:document/:revision/*

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
};

use tolerance_api_core::query::{parse_nist_document_key, require_cmmc_structured, ElementQuery};
use tolerance_api_core::service;

use crate::cmmc::format_response::{wants_toon, FormatResponse};
use crate::cmmc::model::{Element, Relationship};
use crate::cmmc::response::{DataSummary, Family, PaginatedResponse, Requirement, SecurityRequirement};
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

/// Lightweight description of an available document. Defined in core; re-exported
/// here so the OpenAPI spec can reference `crate::cmmc::handler::nist::DocumentInfo`.
pub use tolerance_api_core::service::DocumentInfo;

/// Get list of available documents.
#[utoipa::path(
    get,
    path = "/v1/nist/documents",
    responses(
        (status = 200, description = "List of available documents. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens).",
         content(
             (Vec<DocumentInfo> = "application/json"),
             (String = "text/toon")
         ))
    ),
    tag = "NIST"
)]
pub async fn get_documents(
    State(state): State<CmmcState>,
    headers: HeaderMap,
) -> FormatResponse<Vec<DocumentInfo>> {
    let docs = service::list_nist_documents(&state.available_documents());
    FormatResponse::with_format(docs, wants_toon(&headers))
}

/// Get summary for a specific document+revision.
#[utoipa::path(
    get,
    operation_id = "nist_get_summary",
    path = "/v1/nist/{document}/{revision}/summary",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "Summary retrieved successfully. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens).",
         content(
             (DataSummary = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_summary(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<DataSummary>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    Ok(FormatResponse::with_format(service::summary(&doc), wants_toon(&headers)))
}

/// Get all families.
#[utoipa::path(
    get,
    operation_id = "nist_get_families",
    path = "/v1/nist/{document}/{revision}/families",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of families.",
         content(
             (Vec<Family> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_families(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Family>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let families = service::families(&doc, state.scoring_db(), state.poam_validator());
    Ok(FormatResponse::with_format(families, wants_toon(&headers)))
}

/// Get a specific family by identifier.
#[utoipa::path(
    get,
    operation_id = "nist_get_family",
    path = "/v1/nist/{document}/{revision}/families/{id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("id" = String, Path, description = "Family identifier")
    ),
    responses(
        (status = 200, description = "Family details.",
         content(
             (Family = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Family not found")
    ),
    tag = "NIST"
)]
pub async fn get_family(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Family>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let family = service::family(&doc, state.scoring_db(), state.poam_validator(), &id)?;
    Ok(FormatResponse::with_format(family, wants_toon(&headers)))
}

/// Get all elements.
#[utoipa::path(
    get,
    operation_id = "nist_get_elements",
    path = "/v1/nist/{document}/{revision}/elements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ElementQuery
    ),
    responses(
        (status = 200, description = "Paginated list of elements.",
         content(
             (PaginatedResponse<Element> = "application/json"),
             (String = "text/toon")
         )),
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
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    Ok(FormatResponse::with_format(service::elements(&doc, &query), wants_toon(&headers)))
}

/// Get a specific element by identifier.
#[utoipa::path(
    get,
    operation_id = "nist_get_element",
    path = "/v1/nist/{document}/{revision}/elements/{id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element details.",
         content(
             (Element = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
pub async fn get_element(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Element>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let element = service::element(&doc, &id)?;
    Ok(FormatResponse::with_format(element, wants_toon(&headers)))
}

/// Get all requirements across all families.
#[utoipa::path(
    get,
    operation_id = "nist_get_requirements",
    path = "/v1/nist/{document}/{revision}/requirements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of requirements.",
         content(
             (Vec<Requirement> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Requirement>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let requirements = service::requirements(&doc, state.scoring_db(), state.poam_validator());
    Ok(FormatResponse::with_format(requirements, wants_toon(&headers)))
}

/// Get all security requirements with discussion and assessment text.
#[utoipa::path(
    get,
    operation_id = "nist_get_security_requirements",
    path = "/v1/nist/{document}/{revision}/security-requirements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of security requirements.",
         content(
             (Vec<SecurityRequirement> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_security_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<SecurityRequirement>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let security_requirements =
        service::security_requirements(&doc, state.scoring_db(), state.poam_validator());
    Ok(FormatResponse::with_format(security_requirements, wants_toon(&headers)))
}

/// Get all relationships.
#[utoipa::path(
    get,
    operation_id = "nist_get_relationships",
    path = "/v1/nist/{document}/{revision}/relationships",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of relationships.",
         content(
             (Vec<Relationship> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_relationships(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    Ok(FormatResponse::with_format(service::relationships(&doc), wants_toon(&headers)))
}

/// Get relationships for a specific element.
#[utoipa::path(
    get,
    operation_id = "nist_get_element_relationships",
    path = "/v1/nist/{document}/{revision}/elements/{id}/relationships",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element relationships.",
         content(
             (Vec<Relationship> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let relationships = service::element_relationships(&doc, &id)?;
    Ok(FormatResponse::with_format(relationships, wants_toon(&headers)))
}
