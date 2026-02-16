use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::handler::error::ApiError;
use crate::model::cmmc::{
    DataSummary, Element, ElementType, Family, NistData, Relationship,
    Requirement, SecurityRequirement,
};

/// Shared state for CMMC data
#[derive(Clone)]
pub struct CmmcState {
    pub data: Arc<NistData>,
}

impl CmmcState {
    pub fn new(data: NistData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    /// Load CMMC data from JSON file
    pub fn from_json_file(path: &str) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        let data: NistData = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::new(data))
    }
}

/// Query parameters for filtering elements
#[derive(Debug, Deserialize)]
pub struct ElementQuery {
    /// Filter by element type
    #[serde(rename = "type")]
    pub element_type: Option<String>,
    /// Search in title or text
    pub search: Option<String>,
}

/// Get summary of the NIST 800-171 data
pub async fn get_summary(State(state): State<CmmcState>) -> Json<DataSummary> {
    let elements = &state.data.response.elements;

    let family_count = elements
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::Family)
        .count();

    let requirement_count = elements
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::Requirement)
        .count();

    let security_requirement_count = elements
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::SecurityRequirement)
        .count();

    let summary = DataSummary {
        document: elements.documents.first().cloned().unwrap_or_else(|| {
            crate::model::cmmc::Document {
                doc_identifier: String::new(),
                name: String::new(),
                version: String::new(),
                website: String::new(),
            }
        }),
        family_count,
        requirement_count,
        security_requirement_count,
        relationship_count: elements.relationships.len(),
    };

    Json(summary)
}

/// Get all families
pub async fn get_families(State(state): State<CmmcState>) -> Json<Vec<Family>> {
    let elements = &state.data.response.elements.elements;
    let relationships = &state.data.response.elements.relationships;

    let families: Vec<Family> = elements
        .iter()
        .filter(|e| e.element_type == ElementType::Family)
        .map(|family| {
            let requirements = get_family_requirements(family, elements, relationships);
            Family {
                identifier: family.element_identifier.clone(),
                title: family.title.clone(),
                requirements,
            }
        })
        .collect();

    Json(families)
}

/// Get a specific family by identifier
pub async fn get_family(
    State(state): State<CmmcState>,
    Path(id): Path<String>,
) -> Result<Json<Family>, ApiError> {
    let elements = &state.data.response.elements.elements;
    let relationships = &state.data.response.elements.relationships;

    let family = elements
        .iter()
        .find(|e| e.element_type == ElementType::Family && e.element_identifier == id)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    let requirements = get_family_requirements(family, elements, relationships);

    Ok(Json(Family {
        identifier: family.element_identifier.clone(),
        title: family.title.clone(),
        requirements,
    }))
}

/// Get all elements with optional filtering
pub async fn get_elements(
    State(state): State<CmmcState>,
    Query(query): Query<ElementQuery>,
) -> Json<Vec<Element>> {
    let mut elements: Vec<Element> = state
        .data
        .response
        .elements
        .elements
        .iter()
        .cloned()
        .collect();

    // Filter by element type
    if let Some(ref type_filter) = query.element_type {
        let element_type = match type_filter.to_lowercase().as_str() {
            "family" => Some(ElementType::Family),
            "requirement" => Some(ElementType::Requirement),
            "security_requirement" => Some(ElementType::SecurityRequirement),
            "discussion" => Some(ElementType::Discussion),
            "assessment" => Some(ElementType::Assessment),
            _ => None,
        };

        if let Some(et) = element_type {
            elements.retain(|e| e.element_type == et);
        }
    }

    // Filter by search term
    if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        elements.retain(|e| {
            e.title.to_lowercase().contains(&search_lower)
                || e.text.to_lowercase().contains(&search_lower)
                || e.element_identifier.to_lowercase().contains(&search_lower)
        });
    }

    Json(elements)
}

/// Get a specific element by identifier
pub async fn get_element(
    State(state): State<CmmcState>,
    Path(id): Path<String>,
) -> Result<Json<Element>, ApiError> {
    let element = state
        .data
        .response
        .elements
        .elements
        .iter()
        .find(|e| e.element_identifier == id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    Ok(Json(element))
}

/// Get all requirements
pub async fn get_requirements(State(state): State<CmmcState>) -> Json<Vec<Element>> {
    let requirements: Vec<Element> = state
        .data
        .response
        .elements
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::Requirement)
        .cloned()
        .collect();

    Json(requirements)
}

/// Get all security requirements
pub async fn get_security_requirements(State(state): State<CmmcState>) -> Json<Vec<Element>> {
    let security_requirements: Vec<Element> = state
        .data
        .response
        .elements
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::SecurityRequirement)
        .cloned()
        .collect();

    Json(security_requirements)
}

/// Get relationships for a specific element
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path(id): Path<String>,
) -> Json<Vec<Relationship>> {
    let relationships: Vec<Relationship> = state
        .data
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
    Json(state.data.response.elements.relationships.clone())
}

// Helper functions

fn get_family_requirements(
    family: &Element,
    elements: &[Element],
    relationships: &[Relationship],
) -> Vec<Requirement> {
    // Find requirements that belong to this family (by identifier prefix)
    let family_prefix = &family.element_identifier; // e.g., "03.01"

    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::Requirement
                && e.element_identifier.starts_with(family_prefix)
                && e.element_identifier.len() > family_prefix.len()
        })
        .map(|req| {
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
    // Security requirements have identifiers like "SR-03.01.01.a"
    let req_id = &requirement.element_identifier; // e.g., "03.01.01"
    let sr_prefix = format!("SR-{}", req_id);

    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::SecurityRequirement
                && e.element_identifier.starts_with(&sr_prefix)
        })
        .map(|sr| {
            // Find associated discussion and assessment
            let discussion = find_related_text(elements, &sr.element_identifier, ElementType::Discussion);
            let assessment = find_related_text(elements, &sr.element_identifier, ElementType::Assessment);

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
    // Discussion/Assessment identifiers are typically related to the security requirement
    elements
        .iter()
        .find(|e| {
            e.element_type == element_type
                && e.element_identifier.contains(sr_identifier)
        })
        .map(|e| e.text.clone())
        .filter(|t| !t.is_empty())
}
