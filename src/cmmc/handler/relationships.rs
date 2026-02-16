//! Relationship endpoint handlers

use axum::{
    extract::{Path, State},
    Json,
};

use crate::cmmc::model::Relationship;
use crate::cmmc::state::CmmcState;

/// Get relationships for a specific element
///
/// Complexity: O(r) where r = relationship count
/// TODO: Could be optimized with relationship index if needed
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path(id): Path<String>,
) -> Json<Vec<Relationship>> {
    let relationships: Vec<Relationship> = state
        .data()
        .response
        .elements
        .relationships
        .iter()
        .filter(|r| r.source_element_identifier == id || r.dest_element_identifier == id)
        .cloned()
        .collect();

    Json(relationships)
}

/// Get all relationships
pub async fn get_relationships(State(state): State<CmmcState>) -> Json<Vec<Relationship>> {
    Json(state.data().response.elements.relationships.clone())
}
