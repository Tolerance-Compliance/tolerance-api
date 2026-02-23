use axum::{
    extract::{Path, State},
    Json,
};

use crate::cmmc::model::{Element, ElementType, NistDocumentKey, Relationship};
use crate::cmmc::response::{Family, Requirement, SecurityRequirement};
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::{parse_document_key, parse_level};

/// Get all families - uses type index for O(f) where f = family count (new API)
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/families",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)")
    ),
    responses(
        (status = 200, description = "List of families", body = Vec<Family>),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_families(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
) -> Result<Json<Vec<Family>>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    get_families_impl(state, key)
}

/// Get all families - uses type index for O(f) where f = family count (legacy CMMC API)
#[utoipa::path(
    get,
    operation_id = "cmmc_get_families",
    path = "/api/v1/cmmc/{level}/families",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)")
    ),
    responses(
        (status = 200, description = "List of families", body = Vec<Family>),
        (status = 404, description = "Level not found")
    ),
    tag = "CMMC (Legacy)"
)]
#[allow(deprecated)]
pub async fn get_families_legacy(
    State(state): State<CmmcState>,
    Path(level): Path<String>,
) -> Result<Json<Vec<Family>>, ApiError> {
    use crate::cmmc::model::{CmmcLevel, NistDocument, NistRevision};
    let level = parse_level(&level)?;
    let key = match level {
        CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
        CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
    };
    get_families_impl(state, key)
}

fn get_families_impl(state: CmmcState, key: NistDocumentKey) -> Result<Json<Vec<Family>>, ApiError> {
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let relationships = &state.data(key).unwrap().response.elements.relationships;
    let family_indices = state.index(key).unwrap().get_by_type(ElementType::Family);

    let families: Vec<Family> = family_indices
        .iter()
        .filter_map(|&idx| elements.get(idx))
        .map(|family: &Element| build_family(family, elements, relationships))
        .collect();

    Ok(Json(families))
}

/// Get a specific family by identifier
#[utoipa::path(
    get,
    path = "/v1/nist/{document}/{revision}/families/{id}",
    params(
        ("document" = String, Path, description = "NIST document (sp800-171 or sp800-172)"),
        ("revision" = String, Path, description = "Document revision (r1, r2, r3, or v1)"),
        ("id" = String, Path, description = "Family identifier")
    ),
    responses(
        (status = 200, description = "Family details", body = Family),
        (status = 404, description = "Family not found")
    ),
    tag = "NIST"
)]
pub async fn get_family(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
) -> Result<Json<Family>, ApiError> {
    let key = parse_document_key(&document, &revision)?;
    get_family_impl(state, key, id)
}

/// Get a specific family by identifier - O(1) lookup (legacy CMMC API)
#[utoipa::path(
    get,
    operation_id = "cmmc_get_family",
    path = "/api/v1/cmmc/{level}/families/{id}",
    params(
        ("level" = String, Path, description = "CMMC level (l2 for SP 800-171, l3 for SP 800-172)"),
        ("id" = String, Path, description = "Family identifier")
    ),
    responses(
        (status = 200, description = "Family details", body = Family),
        (status = 404, description = "Family not found")
    ),
    tag = "CMMC (Legacy)"
)]
#[allow(deprecated)]
pub async fn get_family_legacy(
    State(state): State<CmmcState>,
    Path((level, id)): Path<(String, String)>,
) -> Result<Json<Family>, ApiError> {
    use crate::cmmc::model::{CmmcLevel, NistDocument, NistRevision};
    let level = parse_level(&level)?;
    let key = match level {
        CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
        CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
    };
    get_family_impl(state, key, id)
}

fn get_family_impl(state: CmmcState, key: NistDocumentKey, id: String) -> Result<Json<Family>, ApiError> {
    let elements = state.elements(key).ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;
    let relationships = &state.data(key).unwrap().response.elements.relationships;
    let index = state.index(key).unwrap();

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

    let family_prefix_dot = format!("{}.", family_prefix);
    elements
        .iter()
        .filter(|e: &&Element| {
            e.element_type == ElementType::Requirement
                && e.element_identifier.starts_with(&family_prefix_dot)
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
        .find(|e: &&Element| e.element_type == element_type && e.element_identifier == sr_identifier)
        .map(|e: &Element| e.text.clone())
        .filter(|t: &String| !t.is_empty())
}
