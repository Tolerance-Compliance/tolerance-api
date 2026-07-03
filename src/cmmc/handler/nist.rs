//! /v1/nist/:document/:revision/*

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
};

use crate::cmmc::format_response::{FormatResponse, wants_toon};
use crate::cmmc::model::{Document, Element, ElementType, Relationship};
use crate::cmmc::response::{
    DataSummary, Family, PaginatedResponse, Requirement, SecurityRequirement,
};
use crate::cmmc::state::CmmcState;
use crate::handler::error::ApiError;

use super::query::{ElementQuery, parse_nist_document_key, require_cmmc_structured};

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DocumentInfo {
    /// URL path identifier (e.g., "sp800-171/r3")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// NIST document identifier
    pub document: String,
    /// Revision string
    pub revision: String,
}

/// Get list of available documents.
#[utoipa::path(
    get,
    path = "/v1/nist/documents",
    responses(
        (status = 200, description = "List of available documents. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens).",
         content(
             (Vec<DocumentInfo> = "application/json"),
             (String = "text/toon")
         ))
    ),
    tag = "NIST"
)]
pub async fn get_documents(
    State(state): State<CmmcState>,
    headers: HeaderMap,
) -> FormatResponse<Vec<DocumentInfo>> {
    let mut docs: Vec<DocumentInfo> = state
        .available_documents()
        .into_iter()
        .filter_map(|key| {
            // Only include NIST documents in this endpoint
            if matches!(key.source(), crate::cmmc::model::DocumentSource::Nist) {
                Some(DocumentInfo {
                    id: key.to_string(),
                    name: key.display_name(),
                    document: key.document_string(),
                    revision: key.revision_string(),
                })
            } else {
                None
            }
        })
        .collect();

    docs.sort_by(|a, b| a.id.cmp(&b.id));
    FormatResponse::with_format(docs, wants_toon(&headers))
}

/// Get summary for a specific document+revision.
#[utoipa::path(
    get,
    operation_id = "nist_get_summary",
    path = "/v1/nist/{document}/{revision}/summary",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "Summary retrieved successfully. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens).",
         content(
             (DataSummary = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_summary(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<DataSummary>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let summary = DataSummary {
        document: doc.documents.first().cloned().unwrap_or_else(|| Document {
            doc_identifier: String::new(),
            name: String::new(),
            version: String::new(),
            website: String::new(),
        }),
        family_count: doc.index.count_by_type(ElementType::Family),
        requirement_count: doc.index.count_by_type(ElementType::Requirement),
        security_requirement_count: doc.index.count_by_type(ElementType::SecurityRequirement),
        relationship_count: doc.relationships.len(),
    };

    Ok(FormatResponse::with_format(summary, wants_toon(&headers)))
}

/// Get all families.
#[utoipa::path(
    get,
    operation_id = "nist_get_families",
    path = "/v1/nist/{document}/{revision}/families",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of families.",
         content(
             (Vec<Family> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_families(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Family>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let families = doc
        .index
        .get_by_type(ElementType::Family)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|family| {
            build_family(
                family,
                doc.elements,
                state.scoring_db(),
                state.poam_validator(),
            )
        })
        .collect();

    Ok(FormatResponse::with_format(families, wants_toon(&headers)))
}

/// Get a specific family by identifier.
#[utoipa::path(
    get,
    operation_id = "nist_get_family",
    path = "/v1/nist/{document}/{revision}/families/{id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("id" = String, Path, description = "Family identifier")
    ),
    responses(
        (status = 200, description = "Family details.",
         content(
             (Family = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Family not found")
    ),
    tag = "NIST"
)]
pub async fn get_family(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Family>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let idx = doc
        .index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    let family = doc
        .elements
        .get(idx)
        .filter(|e| e.element_type == ElementType::Family)
        .ok_or_else(|| ApiError::NotFound(format!("Family '{}' not found", id)))?;

    Ok(FormatResponse::with_format(
        build_family(
            family,
            doc.elements,
            state.scoring_db(),
            state.poam_validator(),
        ),
        wants_toon(&headers),
    ))
}

/// Get all elements.
#[utoipa::path(
    get,
    operation_id = "nist_get_elements",
    path = "/v1/nist/{document}/{revision}/elements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ElementQuery
    ),
    responses(
        (status = 200, description = "Paginated list of elements.",
         content(
             (PaginatedResponse<Element> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_elements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    Query(query): Query<ElementQuery>,
    headers: HeaderMap,
) -> Result<FormatResponse<PaginatedResponse<Element>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let filtered: Vec<usize> = match (query.parse_element_type(), &query.search) {
        (Some(et), Some(term)) => doc.index.search(term, Some(et)),
        (Some(et), None) => doc.index.get_by_type(et).to_vec(),
        (None, Some(term)) => doc.index.search(term, None),
        (None, None) => (0..doc.elements.len()).collect(),
    };

    let total = filtered.len();
    let offset = query.offset();
    let limit = query.limit();

    let items = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter_map(|idx| doc.elements.get(idx))
        .cloned()
        .collect();

    Ok(FormatResponse::with_format(
        PaginatedResponse {
            data: items,
            total,
            offset,
            limit,
            has_more: offset + limit < total,
        },
        wants_toon(&headers),
    ))
}

/// Get a specific element by identifier.
#[utoipa::path(
    get,
    operation_id = "nist_get_element",
    path = "/v1/nist/{document}/{revision}/elements/{id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element details.",
         content(
             (Element = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
pub async fn get_element(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Element>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let element = doc
        .index
        .get_by_identifier(&id)
        .and_then(|idx| doc.elements.get(idx))
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    Ok(FormatResponse::with_format(element, wants_toon(&headers)))
}

/// Get all requirements across all families.
#[utoipa::path(
    get,
    operation_id = "nist_get_requirements",
    path = "/v1/nist/{document}/{revision}/requirements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of requirements.",
         content(
             (Vec<Requirement> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Requirement>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let requirements = doc
        .index
        .get_by_type(ElementType::Requirement)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|req| {
            build_requirement(
                req,
                doc.elements,
                state.scoring_db(),
                state.poam_validator(),
            )
        })
        .collect();

    Ok(FormatResponse::with_format(
        requirements,
        wants_toon(&headers),
    ))
}

/// Get all security requirements with discussion and assessment text.
#[utoipa::path(
    get,
    operation_id = "nist_get_security_requirements",
    path = "/v1/nist/{document}/{revision}/security-requirements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of security requirements.",
         content(
             (Vec<SecurityRequirement> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_security_requirements(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<SecurityRequirement>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let security_requirements = doc
        .index
        .get_by_type(ElementType::SecurityRequirement)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|sr| {
            build_security_requirement(sr, doc.elements, state.scoring_db(), state.poam_validator())
        })
        .collect();

    Ok(FormatResponse::with_format(
        security_requirements,
        wants_toon(&headers),
    ))
}

/// Get all relationships.
#[utoipa::path(
    get,
    operation_id = "nist_get_relationships",
    path = "/v1/nist/{document}/{revision}/relationships",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)")
    ),
    responses(
        (status = 200, description = "List of relationships.",
         content(
             (Vec<Relationship> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
pub async fn get_relationships(
    State(state): State<CmmcState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    Ok(FormatResponse::with_format(
        doc.relationships.to_vec(),
        wants_toon(&headers),
    ))
}

/// Get relationships for a specific element.
#[utoipa::path(
    get,
    operation_id = "nist_get_element_relationships",
    path = "/v1/nist/{document}/{revision}/elements/{id}/relationships",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`, `sp800-171a`, `sp800-172`, or `sp800-172a`"),
        ("revision" = String, Path, description = "Document revision: `r2`, `r3` (171/171a), `v1` (172/172a)"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element relationships.",
         content(
             (Vec<Relationship> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
pub async fn get_element_relationships(
    State(state): State<CmmcState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let doc = state
        .get_document(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    doc.index
        .get_by_identifier(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Element '{}' not found", id)))?;

    let relationships = doc
        .relationships
        .iter()
        .filter(|r| r.source_element_identifier == id || r.dest_element_identifier == id)
        .cloned()
        .collect();

    Ok(FormatResponse::with_format(
        relationships,
        wants_toon(&headers),
    ))
}

// These are `pub(crate)` so the legacy CMMC handlers in families.rs can reuse
// them without duplicating the logic.

use crate::cmmc::poam::PoamValidator;
use crate::cmmc::scoring::ScoringDatabase;

pub(crate) fn build_family(
    family: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Family {
    Family {
        identifier: family.element_identifier.clone(),
        title: family.title.clone(),
        requirements: get_family_requirements(family, elements, scoring_db, poam_validator),
    }
}

fn get_family_requirements(
    family: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Vec<Requirement> {
    let prefix = format!("{}.", family.element_identifier);
    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::Requirement && e.element_identifier.starts_with(&prefix)
        })
        .map(|req| build_requirement(req, elements, scoring_db, poam_validator))
        .collect()
}

pub(crate) fn build_requirement(
    req: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Requirement {
    let score = scoring_db.get_score(&req.element_identifier).cloned();
    let poam_validation = Some(poam_validator.validate(&req.element_identifier));

    Requirement {
        identifier: req.element_identifier.clone(),
        title: req.title.clone(),
        text: req.text.clone(),
        security_requirements: get_security_requirements_for(
            req,
            elements,
            scoring_db,
            poam_validator,
        ),
        score,
        poam_validation,
    }
}

fn get_security_requirements_for(
    req: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Vec<SecurityRequirement> {
    let prefix = format!("SR-{}", req.element_identifier);
    elements
        .iter()
        .filter(|e| {
            e.element_type == ElementType::SecurityRequirement
                && e.element_identifier.starts_with(&prefix)
        })
        .map(|sr| build_security_requirement(sr, elements, scoring_db, poam_validator))
        .collect()
}

pub(crate) fn build_security_requirement(
    sr: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> SecurityRequirement {
    // For security requirements, we check if the parent requirement has scoring
    // Security requirements inherit the parent requirement's score
    let parent_id = extract_parent_requirement_id(&sr.element_identifier);
    let score = parent_id
        .as_ref()
        .and_then(|id| scoring_db.get_score(id))
        .cloned();
    let poam_validation = parent_id.as_ref().map(|id| poam_validator.validate(id));

    SecurityRequirement {
        identifier: sr.element_identifier.clone(),
        title: sr.title.clone(),
        text: sr.text.clone(),
        discussion: find_related_text(elements, &sr.element_identifier, ElementType::Discussion),
        assessment: find_related_text(elements, &sr.element_identifier, ElementType::Assessment),
        score,
        poam_validation,
    }
}

fn find_related_text(elements: &[Element], id: &str, element_type: ElementType) -> Option<String> {
    elements
        .iter()
        .find(|e| e.element_type == element_type && e.element_identifier == id)
        .map(|e| e.text.clone())
        .filter(|t| !t.is_empty())
}

/// Extract parent requirement ID from a security requirement identifier
/// e.g., "SR-03.01.01.a" -> Some("03.01.01")
fn extract_parent_requirement_id(sr_id: &str) -> Option<String> {
    if let Some(stripped) = sr_id.strip_prefix("SR-") {
        // Find the last dot and take everything before it, or the whole thing if no dot
        let parts: Vec<&str> = stripped.split('.').collect();
        if parts.len() >= 3 {
            // Take first 3 parts (e.g., "03", "01", "01")
            Some(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
        } else {
            Some(stripped.to_string())
        }
    } else {
        None
    }
}
