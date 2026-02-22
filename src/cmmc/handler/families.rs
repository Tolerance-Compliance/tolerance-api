use axum::{
    extract::{Path, State},
    Json,
};

use crate::cmmc::model::{Element, ElementType, Relationship};
use crate::cmmc::response::{Family, Requirement, SecurityRequirement};
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::parse_level;

/// Get all families - uses type index for O(f) where f = family count
pub async fn get_families(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<Vec<Family>>, ApiError> {
    let level = parse_level(&level)?;
    let elements = state.elements(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;
    let relationships = &state.data(level).unwrap().response.elements.relationships;
    let family_indices = state.index(level).unwrap().get_by_type(ElementType::Family);

    let families: Vec<Family> = family_indices
        .iter()
        .filter_map(|&idx| elements.get(idx))
        .map(|family: &Element| build_family(family, elements, relationships))
        .collect();

    Ok(Json(families))
}

/// Get a specific family by identifier - O(1) lookup
pub async fn get_family(
    State(state): State<CmmcState>,
    Path((level, id)): Path<(String, String)>,
) -> Result<Json<Family>, ApiError> {
    let level = parse_level(&level)?;
    let elements = state.elements(level).ok_or_else(|| ApiError::NotFound(format!("Level {} not loaded", level)))?;
    let relationships = &state.data(level).unwrap().response.elements.relationships;
    let index = state.index(level).unwrap();

    let idx: usize = index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    let family = elements
        .get(idx)
        .filter(|e: &&Element| e.element_type == ElementType::Family)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    Ok(Json(build_family(family, elements, relationships)))
}

fn build_family(family: &Element, elements: &[Element], relationships: &[Relationship]) -> Family {
    let requirements: Vec<Requirement> = get_family_requirements(family, elements, relationships);
    Family {
        identifier: family.element_identifier.clone(),
        title: family.title.clone(),
        requirements,
    }
}

fn get_family_requirements(
    family: &Element,
    elements: &[Element],
    relationships: &[Relationship],
) -> Vec<Requirement> {
    let family_prefix = &family.element_identifier;

    elements
        .iter()
        .filter(|e: &&Element| {
            e.element_type == ElementType::Requirement
                && e.element_identifier.starts_with(family_prefix)
                && e.element_identifier.len() > family_prefix.len()
        })
        .map(|req: &Element| {
            let security_requirements =
                get_requirement_security_requirements(req, elements, relationships);
            Requirement {
                identifier: req.element_identifier.clone(),
                title: req.title.clone(),
                text: req.text.clone(),
                security_requirements,
            }
        })
        .collect()
}

fn get_requirement_security_requirements(
    requirement: &Element,
    elements: &[Element],
    _relationships: &[Relationship],
) -> Vec<SecurityRequirement> {
    let req_id: &String = &requirement.element_identifier;
    let sr_prefix: String = format!("SR-{}", req_id);

    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::SecurityRequirement
                && e.element_identifier.starts_with(&sr_prefix)
        })
        .map(|sr: &Element| {
            let discussion =
                find_related_text(elements, &sr.element_identifier, ElementType::Discussion);
            let assessment =
                find_related_text(elements, &sr.element_identifier, ElementType::Assessment);

            SecurityRequirement {
                identifier: sr.element_identifier.clone(),
                title: sr.title.clone(),
                text: sr.text.clone(),
                discussion,
                assessment,
            }
        })
        .collect()
}

fn find_related_text(
    elements: &[Element],
    sr_identifier: &str,
    element_type: ElementType,
) -> Option<String> {
    elements
        .iter()
        .find(|e: &&Element| e.element_type == element_type && e.element_identifier.contains(sr_identifier))
        .map(|e: &Element| e.text.clone())
        .filter(|t: &String| !t.is_empty())
}
