//! POA&M validation endpoints

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};

use tolerance_api_core::query::parse_nist_document_key;
use tolerance_api_core::service;

use crate::cmmc::format_response::{wants_toon, FormatResponse};
use crate::cmmc::poam::PoamValidation;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

/// Request/response bodies are defined in core; re-exported here so the OpenAPI
/// spec can reference `crate::cmmc::handler::poam::BatchValidation*`.
pub use tolerance_api_core::service::{BatchValidationRequest, BatchValidationResponse};

/// Validate whether a single requirement can be added to POA&M
#[utoipa::path(
    get,
    operation_id = "validate_poam_requirement",
    path = "/v1/nist/{document}/{revision}/poam/validate/{requirement_id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("requirement_id" = String, Path, description = "Requirement identifier (e.g., '03.01.01')")
    ),
    responses(
        (status = 200, description = "POA&M validation result. Send `Accept: text/toon` for LLM-optimized output.",
         content(
             (PoamValidation = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "POA&M"
)]
pub async fn validate_poam_requirement(
    State(state): State<CmmcState>,
    Path((document, revision, requirement_id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<PoamValidation>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;

    // Verify the document is loaded.
    state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let validation = service::validate_poam_requirement(state.poam_validator(), &requirement_id);
    Ok(FormatResponse::with_format(validation, wants_toon(&headers)))
}

/// Validate multiple requirements for POA&M eligibility
#[utoipa::path(
    post,
    operation_id = "validate_poam_batch",
    path = "/v1/nist/{document}/{revision}/poam/validate",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    request_body = BatchValidationRequest,
    responses(
        (status = 200, description = "Batch validation results. Send `Accept: text/toon` for LLM-optimized output.",
         content(
             (BatchValidationResponse = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "POA&M"
)]
pub async fn validate_poam_batch(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<BatchValidationRequest>,
) -> Result<FormatResponse<BatchValidationResponse>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;

    // Verify the document is loaded.
    state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let response = service::validate_poam_batch(state.poam_validator(), &request.requirement_ids);
    Ok(FormatResponse::with_format(response, wants_toon(&headers)))
}

/// Get all requirements that cannot be added to POA&M
#[utoipa::path(
    get,
    operation_id = "get_non_eligible_requirements",
    path = "/v1/nist/{document}/{revision}/poam/non-eligible",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of requirement IDs that cannot be added to POA&M. Send `Accept: text/toon` for LLM-optimized output.",
         content(
             (Vec<String> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "POA&M"
)]
pub async fn get_non_eligible_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<String>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;

    // Verify the document is loaded.
    state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let non_eligible = service::non_eligible(state.poam_validator());
    Ok(FormatResponse::with_format(non_eligible, wants_toon(&headers)))
}
