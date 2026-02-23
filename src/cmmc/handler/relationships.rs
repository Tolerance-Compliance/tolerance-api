//! Relationship endpoint handlers (legacy CMMC API)

use axum::{
    extract::{Path, State},
    Json,
};

use crate::cmmc::model::{NistDocument, NistDocumentKey, NistRevision, Relationship};
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::parse_level;

/// Get relationships for a specific element (legacy CMMC API)
#[utoipa::path(
    get,
    path = "/api/v1/cmmc/{level}/elements/{id}/relationships",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "List of relationships for the element", body = Vec<Relationship>),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC (Legacy)"
)]
#[allow(deprecated)]
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path((level, id)): Path<(String, String)>,
) -> Result<Json<Vec<Relationship>>, ApiError> {
    use crate::cmmc::model::CmmcLevel;
    let level = parse_level(&level)?;
    let key = match level {
        CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
        CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
    };
    let data = state.data(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let relationships: Vec<Relationship> = data
        .response
        .elements
        .relationships
        .iter()
        .filter(|r: &&Relationship| r.source_element_identifier == id || r.dest_element_identifier == id)
        .cloned()
        .collect();

    Ok(Json(relationships))
}

/// Get all relationships (legacy CMMC API)
#[utoipa::path(
    get,
    path = "/api/v1/cmmc/{level}/relationships",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)")
    ),
    responses(
        (status = 200, description = "List of all relationships", body = Vec<Relationship>),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC (Legacy)"
)]
#[allow(deprecated)]
pub async fn get_relationships(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<Vec<Relationship>>, ApiError> {
    use crate::cmmc::model::CmmcLevel;
    let level = parse_level(&level)?;
    let key = match level {
        CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
        CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
    };
    let data = state.data(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    Ok(Json(data.response.elements.relationships.clone()))
}
