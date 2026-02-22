//! Relationship endpoint handlers

use axum::{
    extract::{Path, State},
    Json,
};

use crate::cmmc::model::Relationship;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::parse_level;

/// Get relationships for a specific element
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path((level, id)): Path<(String, String)>,
) -> Result<Json<Vec<Relationship>>, ApiError> {
    let level = parse_level(&level)?;
    let data = state.data(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;

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

/// Get all relationships
pub async fn get_relationships(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<Vec<Relationship>>, ApiError> {
    let level = parse_level(&level)?;
    let data = state.data(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;

    Ok(Json(data.response.elements.relationships.clone()))
}
