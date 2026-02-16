//! Element endpoint handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::cmmc::model::{Element, ElementType};
use crate::cmmc::response::PaginatedResponse;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::ElementQuery;

/// Get all elements with optional filtering and pagination
///
/// Complexity:
/// - No search/type filter: O(limit)
/// - Type filter only: O(type_count), paginated
/// - Search: O(1) for exact word match via inverted index, O(n) for substring fallback
pub async fn get_elements(
    State(state): State<CmmcState>,
    Query(query): Query<ElementQuery>,
) -> Json<PaginatedResponse<Element>> {
    let elements = state.elements();
    let limit = query.limit();
    let offset = query.offset();
    let type_filter = query.parse_element_type();

    // Determine which indices to return
    let matching_indices: Vec<usize> = if let Some(ref search_term) = query.search {
        // Use search index - O(1) for word match, O(n) for substring
        state.index().search(search_term, type_filter)
    } else if let Some(et) = type_filter {
        // Type filter only - O(1) lookup
        state.index().get_by_type(et).to_vec()
    } else {
        // No filter - return all indices
        (0..elements.len()).collect()
    };

    let total = matching_indices.len();
    let has_more = offset + limit < total;

    // Apply pagination and collect results (only clone what we return)
    let data: Vec<Element> = matching_indices
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|idx| elements.get(idx).cloned())
        .collect();

    Json(PaginatedResponse {
        data,
        total,
        limit,
        offset,
        has_more,
    })
}

/// Get a specific element by identifier - O(1) lookup
pub async fn get_element(
    State(state): State<CmmcState>,
    Path(id): Path<String>,
) -> Result<Json<Element>, ApiError> {
    // O(1) lookup via index
    let idx = state
        .index()
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let element = state
        .get_element(idx)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    Ok(Json(element.clone()))
}

/// Get all requirements - O(r) where r = requirement count
pub async fn get_requirements(State(state): State<CmmcState>) -> Json<Vec<Element>> {
    let elements = state.elements();
    let indices = state.index().get_by_type(ElementType::Requirement);

    let requirements: Vec<Element> = indices
        .iter()
        .filter_map(|&idx| elements.get(idx).cloned())
        .collect();

    Json(requirements)
}

/// Get all security requirements - O(s) where s = security requirement count
pub async fn get_security_requirements(State(state): State<CmmcState>) -> Json<Vec<Element>> {
    let elements = state.elements();
    let indices = state.index().get_by_type(ElementType::SecurityRequirement);

    let security_requirements: Vec<Element> = indices
        .iter()
        .filter_map(|&idx| elements.get(idx).cloned())
        .collect();

    Json(security_requirements)
}
