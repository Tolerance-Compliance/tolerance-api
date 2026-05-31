//! Framework-agnostic endpoint logic.
//!
//! Each function takes an already-resolved [`DocumentContext`] (plus the scoring
//! database / POA&M validator where needed) and returns a serializable response
//! type or a [`CoreError`]. The native handlers and the Worker handlers both call
//! these, which is what keeps their output identical. Transport concerns
//! (parsing params, resolving documents, encoding) stay in the caller.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::document::DocumentContext;
use crate::error::CoreError;
use crate::model::{Document, DocumentKey, DocumentSource, Element, ElementType, Relationship};
use crate::poam::{PoamEligibility, PoamValidation, PoamValidator};
use crate::response::{DataSummary, Family, PaginatedResponse, Requirement, SecurityRequirement};
use crate::scoring::ScoringDatabase;

/// Lightweight description of an available document (for `/v1/nist/documents`).
#[derive(Debug, Serialize, ToSchema)]
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

/// Build the sorted list of NIST documents from a set of available keys.
///
/// FAR documents are filtered out (they have their own `/v1/far/...` surface).
pub fn list_nist_documents(keys: &[DocumentKey]) -> Vec<DocumentInfo> {
    let mut docs: Vec<DocumentInfo> = keys
        .iter()
        .filter(|key| matches!(key.source(), DocumentSource::Nist))
        .map(|key| DocumentInfo {
            id: key.to_string(),
            name: key.display_name(),
            document: key.document_string(),
            revision: key.revision_string(),
        })
        .collect();

    docs.sort_by(|a, b| a.id.cmp(&b.id));
    docs
}

/// Summary statistics for a document.
pub fn summary(doc: &DocumentContext) -> DataSummary {
    DataSummary {
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
    }
}

/// All families, each with its nested requirements.
pub fn families(
    doc: &DocumentContext,
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Vec<Family> {
    doc.index
        .get_by_type(ElementType::Family)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|family| build_family(family, doc.elements, scoring_db, poam_validator))
        .collect()
}

/// A single family by identifier.
pub fn family(
    doc: &DocumentContext,
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
    id: &str,
) -> Result<Family, CoreError> {
    let idx = doc
        .index
        .get_by_identifier(id)
        .ok_or_else(|| CoreError::NotFound(format!("Family '{}' not found", id)))?;

    let family = doc
        .elements
        .get(idx)
        .filter(|e| e.element_type == ElementType::Family)
        .ok_or_else(|| CoreError::NotFound(format!("Family '{}' not found", id)))?;

    Ok(build_family(family, doc.elements, scoring_db, poam_validator))
}

/// Paginated, optionally filtered list of elements.
pub fn elements(doc: &DocumentContext, query: &crate::query::ElementQuery) -> PaginatedResponse<Element> {
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

    PaginatedResponse {
        data: items,
        total,
        offset,
        limit,
        has_more: offset + limit < total,
    }
}

/// A single element by identifier.
pub fn element(doc: &DocumentContext, id: &str) -> Result<Element, CoreError> {
    doc.index
        .get_by_identifier(id)
        .and_then(|idx| doc.elements.get(idx))
        .cloned()
        .ok_or_else(|| CoreError::NotFound(format!("Element '{}' not found", id)))
}

/// All requirements across all families.
pub fn requirements(
    doc: &DocumentContext,
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Vec<Requirement> {
    doc.index
        .get_by_type(ElementType::Requirement)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|req| build_requirement(req, doc.elements, scoring_db, poam_validator))
        .collect()
}

/// All security requirements with discussion and assessment text.
pub fn security_requirements(
    doc: &DocumentContext,
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> Vec<SecurityRequirement> {
    doc.index
        .get_by_type(ElementType::SecurityRequirement)
        .iter()
        .filter_map(|&idx| doc.elements.get(idx))
        .map(|sr| build_security_requirement(sr, doc.elements, scoring_db, poam_validator))
        .collect()
}

/// All relationships in the document.
pub fn relationships(doc: &DocumentContext) -> Vec<Relationship> {
    doc.relationships.to_vec()
}

/// Relationships touching a specific element (as source or destination).
pub fn element_relationships(
    doc: &DocumentContext,
    id: &str,
) -> Result<Vec<Relationship>, CoreError> {
    doc.index
        .get_by_identifier(id)
        .ok_or_else(|| CoreError::NotFound(format!("Element '{}' not found", id)))?;

    Ok(doc
        .relationships
        .iter()
        .filter(|r| r.source_element_identifier == id || r.dest_element_identifier == id)
        .cloned()
        .collect())
}

// ---------------------------------------------------------------------------
// POA&M
// ---------------------------------------------------------------------------

/// Request body for batch POA&M validation.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchValidationRequest {
    /// List of requirement identifiers to validate.
    pub requirement_ids: Vec<String>,
}

/// Response for batch POA&M validation.
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchValidationResponse {
    /// Validation results for each requirement.
    pub validations: Vec<PoamValidation>,
    /// Total number of requirements validated.
    pub total: usize,
    /// Number of eligible requirements.
    pub eligible_count: usize,
    /// Number of not eligible requirements.
    pub not_eligible_count: usize,
    /// Number of conditional requirements.
    pub conditional_count: usize,
}

/// Validate a single requirement for POA&M eligibility.
pub fn validate_poam_requirement(poam_validator: &PoamValidator, requirement_id: &str) -> PoamValidation {
    poam_validator.validate(requirement_id)
}

/// Validate a batch of requirements, returning per-requirement results and counts.
pub fn validate_poam_batch(
    poam_validator: &PoamValidator,
    requirement_ids: &[String],
) -> BatchValidationResponse {
    let validations = poam_validator.validate_batch(requirement_ids);

    let eligible_count = validations
        .iter()
        .filter(|v| matches!(v.eligibility, PoamEligibility::Eligible))
        .count();
    let not_eligible_count = validations
        .iter()
        .filter(|v| matches!(v.eligibility, PoamEligibility::NotEligible))
        .count();
    let conditional_count = validations
        .iter()
        .filter(|v| matches!(v.eligibility, PoamEligibility::Conditional))
        .count();

    BatchValidationResponse {
        total: validations.len(),
        eligible_count,
        not_eligible_count,
        conditional_count,
        validations,
    }
}

/// All requirement IDs that cannot be added to a POA&M.
pub fn non_eligible(poam_validator: &PoamValidator) -> Vec<String> {
    poam_validator.get_non_eligible_requirements()
}

// ---------------------------------------------------------------------------
// Builders (shared by the NIST and FAR endpoints)
// ---------------------------------------------------------------------------

pub fn build_family(
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
        .filter(|e| e.element_type == ElementType::Requirement && e.element_identifier.starts_with(&prefix))
        .map(|req| build_requirement(req, elements, scoring_db, poam_validator))
        .collect()
}

pub fn build_requirement(
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
        security_requirements: get_security_requirements_for(req, elements, scoring_db, poam_validator),
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
        .filter(|e| e.element_type == ElementType::SecurityRequirement && e.element_identifier.starts_with(&prefix))
        .map(|sr| build_security_requirement(sr, elements, scoring_db, poam_validator))
        .collect()
}

pub fn build_security_requirement(
    sr: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> SecurityRequirement {
    // Security requirements inherit the parent requirement's score.
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
        let parts: Vec<&str> = stripped.split('.').collect();
        if parts.len() >= 3 {
            Some(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
        } else {
            Some(stripped.to_string())
        }
    } else {
        None
    }
}
