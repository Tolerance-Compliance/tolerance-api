//! Pure assembly of API response shapes (Family / Requirement / SecurityRequirement)
//! from raw `Element`s plus scoring and POA&M data.
//!
//! This logic is intentionally free of `axum`/`worker` dependencies so it can run
//! both in the offline ingest binary (native) and, if ever needed, on the Worker.

use super::model::{Element, ElementType};
use super::poam::PoamValidator;
use super::response::{Family, Requirement, SecurityRequirement};
use super::scoring::ScoringDatabase;

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
        .filter(|e| {
            e.element_type == ElementType::Requirement
                && e.element_identifier.starts_with(&prefix)
        })
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

pub fn build_security_requirement(
    sr: &Element,
    elements: &[Element],
    scoring_db: &ScoringDatabase,
    poam_validator: &PoamValidator,
) -> SecurityRequirement {
    // Security requirements inherit their parent requirement's score.
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
