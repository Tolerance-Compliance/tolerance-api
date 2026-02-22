//! Element endpoint handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::cmmc::model::{Element, ElementType};
use crate::cmmc::response::PaginatedResponse;
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::{parse_level, ElementQuery};

/// Get all elements with optional filtering and pagination
#[utoipa::path(
    get,
    path = "/api/v1/cmmc/{level}/elements",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)"),
        ElementQuery
    ),
    responses(
        (status = 200, description = "Paginated list of elements", body = PaginatedResponse<Element>),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC"
)]
pub async fn get_elements(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
    Query(query): Query<ElementQuery>,
) -> Result<Json<PaginatedResponse<Element>>, ApiError> {
    let level = parse_level(&level)?;
    let elements = state.elements(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;
    let index = state.index(level).unwrap();
    let limit = query.limit();
    let offset = query.offset();
    let type_filter = query.parse_element_type();

    let matching_indices: Vec<usize> = if let Some(ref search_term) = query.search {
        index.search(search_term, type_filter)
    } else if let Some(et) = type_filter {
        index.get_by_type(et).to_vec()
    } else {
        (0..elements.len()).collect()
    };

    let total = matching_indices.len();
    let has_more = offset + limit < total;

    let data: Vec<Element> = matching_indices
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|idx| elements.get(idx).cloned())
        .collect();

    Ok(Json(PaginatedResponse {
        data,
        total,
        limit,
        offset,
        has_more,
    }))
}

/// Get a specific element by identifier - O(1) lookup
#[utoipa::path(
    get,
    path = "/api/v1/cmmc/{level}/elements/{id}",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element details", body = Element),
        (status = 404, description = "Element not found")
    ),
    tag = "CMMC"
)]
pub async fn get_element(
    State(state): State<CmmcState>,
    Path((level, id)): Path<(String, String)>,
) -> Result<Json<Element>, ApiError> {
    let level = parse_level(&level)?;
    let index = state.index(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;

    let idx = index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let element = state
        .get_element(level, idx)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    Ok(Json(element.clone()))
}

/// Get all requirements
#[utoipa::path(
    get,
    path = "/api/v1/cmmc/{level}/requirements",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)")
    ),
    responses(
        (status = 200, description = "List of requirements", body = Vec<Element>),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC"
)]
pub async fn get_requirements(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<Vec<Element>>, ApiError> {
    let level = parse_level(&level)?;
    let elements = state.elements(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;
    let index = state.index(level).unwrap();
    let indices = index.get_by_type(ElementType::Requirement);

    let requirements: Vec<Element> = indices
        .iter()
        .filter_map(|&idx| elements.get(idx).cloned())
        .collect();

    Ok(Json(requirements))
}

/// Get all security requirements
#[utoipa::path(
    get,
    path = "/api/v1/cmmc/{level}/security-requirements",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)")
    ),
    responses(
        (status = 200, description = "List of security requirements", body = Vec<Element>),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC"
)]
pub async fn get_security_requirements(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<Vec<Element>>, ApiError> {
    let level = parse_level(&level)?;
    let elements = state.elements(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;
    let index = state.index(level).unwrap();
    let indices = index.get_by_type(ElementType::SecurityRequirement);

    let security_requirements: Vec<Element> = indices
        .iter()
        .filter_map(|&idx| elements.get(idx).cloned())
        .collect();

    Ok(Json(security_requirements))
}
