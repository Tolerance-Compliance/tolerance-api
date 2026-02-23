//! Summary endpoint handler (legacy CMMC API)

use axum::{extract::{Path, State}, Json};

use crate::cmmc::model::{Document, ElementType, NistDocument, NistDocumentKey, NistRevision};
use crate::cmmc::response::DataSummary;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::parse_level;

/// Get a summary of the dataset for a given CMMC level - O(1) using pre-computed counts (legacy API)
#[utoipa::path(
    get,
    operation_id = "cmmc_get_summary",
    path = "/api/v1/cmmc/{level}/summary",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)")
    ),
    responses(
        (status = 200, description = "Summary retrieved successfully", body = DataSummary),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC (Legacy)"
)]
#[allow(deprecated)]
pub async fn get_summary(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<DataSummary>, ApiError> {
    use crate::cmmc::model::CmmcLevel;
    let level = parse_level(&level)?;
    let key = match level {
        CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
        CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
    };
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
