//! POA&M validation endpoints

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::cmmc::format_response::{FormatResponse, wants_toon};
use crate::cmmc::poam::PoamValidation;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::parse_nist_document_key;

/// Request body for batch POA&M validation
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchValidationRequest {
    /// List of requirement identifiers to validate
    pub requirement_ids: Vec<String>,
}

/// Response for batch POA&M validation
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchValidationResponse {
    /// Validation results for each requirement
    pub validations: Vec<PoamValidation>,
    /// Total number of requirements validated
    pub total: usize,
    /// Number of eligible requirements
    pub eligible_count: usize,
    /// Number of not eligible requirements
    pub not_eligible_count: usize,
    /// Number of conditional requirements
    pub conditional_count: usize,
}

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

    // Verify the document is loaded
    state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let validation = state.poam_validator().validate(&requirement_id);

    Ok(FormatResponse::with_format(
        validation,
        wants_toon(&headers),
    ))
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

    // Verify the document is loaded
    state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let validations = state
        .poam_validator()
        .validate_batch(&request.requirement_ids);

    let eligible_count = validations
        .iter()
        .filter(|v| matches!(v.eligibility, crate::cmmc::poam::PoamEligibility::Eligible))
        .count();

    let not_eligible_count = validations
        .iter()
        .filter(|v| {
            matches!(
                v.eligibility,
                crate::cmmc::poam::PoamEligibility::NotEligible
            )
        })
        .count();

    let conditional_count = validations
        .iter()
        .filter(|v| {
            matches!(
                v.eligibility,
                crate::cmmc::poam::PoamEligibility::Conditional
            )
        })
        .count();

    let response = BatchValidationResponse {
        total: validations.len(),
        eligible_count,
        not_eligible_count,
        conditional_count,
        validations,
    };

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

    // Verify the document is loaded
    state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let non_eligible = state.poam_validator().get_non_eligible_requirements();

    Ok(FormatResponse::with_format(
        non_eligible,
        wants_toon(&headers),
    ))
}
