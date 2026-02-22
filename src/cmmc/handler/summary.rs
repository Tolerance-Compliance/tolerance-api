//! Summary endpoint handler

use axum::{extract::{Path, State}, Json};

use crate::cmmc::model::{Document, ElementType};
use crate::cmmc::response::DataSummary;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::parse_level;

/// Get summary of the dataset for a given CMMC level - O(1) using pre-computed counts
pub async fn get_summary(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<DataSummary>, ApiError> {
    let level = parse_level(&level)?;
    let data = state.data(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;
    let index = state.index(level).unwrap();

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
