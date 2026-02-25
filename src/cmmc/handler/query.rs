//! Query parameter types and path-parsing helpers

use serde::Deserialize;

use crate::cmmc::model::{ElementType, NistDocument, NistDocumentKey, NistRevision};
use crate::handler::error::ApiError;

/// Parse document and revision path segments into a `NistDocumentKey`
pub fn parse_document_key(document: &str, revision: &str) -> Result<NistDocumentKey, ApiError> {
    let doc = document
        .parse::<NistDocument>()
        .map_err(|e| ApiError::BadRequest(e))?;
    let rev = revision
        .parse::<NistRevision>()
        .map_err(|e| ApiError::BadRequest(e))?;

    match (doc, rev) {
        (NistDocument::Sp800171, NistRevision::V1) => {
            return Err(ApiError::BadRequest(
                "SP 800-171 uses revisions, not versions. Use r1, r2, or r3 (e.g. /sp800-171/r3).".to_string(),
            ));
        }
        (NistDocument::Sp800172, NistRevision::Rev1 | NistRevision::Rev2 | NistRevision::Rev3) => {
            return Err(ApiError::BadRequest(
                "SP 800-172 uses versions, not revisions. Use v1 (e.g. /sp800-172/v1).".to_string(),
            ));
        }
        _ => {}
    }

    Ok(NistDocumentKey::new(doc, rev))
}

/// Query parameters for filtering elements with pagination
#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ElementQuery {
    /// Filter by element type
    #[serde(rename = "type")]
    pub element_type: Option<String>,
    /// Search in title or text
    pub search: Option<String>,
    /// Maximum number of results (default: 100, max: 1000)
    pub limit: Option<usize>,
    /// Offset for pagination (default: 0)
    pub offset: Option<usize>,
}

impl ElementQuery {
    pub const DEFAULT_LIMIT: usize = 100;
    pub const MAX_LIMIT: usize = 1000;

    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(Self::DEFAULT_LIMIT).min(Self::MAX_LIMIT)
    }

    pub fn offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }

    pub fn parse_element_type(&self) -> Option<ElementType> {
        self.element_type.as_ref().and_then(|t| {
            match t.to_lowercase().as_str() {
                "family"               => Some(ElementType::Family),
                "requirement"          => Some(ElementType::Requirement),
                "security_requirement" => Some(ElementType::SecurityRequirement),
                "discussion"           => Some(ElementType::Discussion),
                "assessment"           => Some(ElementType::Assessment),
                "adversary_effect"     => Some(ElementType::AdversaryEffect),
                "protection_strategy"  => Some(ElementType::ProtectionStrategy),
                "effect"               => Some(ElementType::Effect),
                "tactic"               => Some(ElementType::Tactic),
                "impact"               => Some(ElementType::Impact),
                "expected_result"      => Some(ElementType::ExpectedResult),
                "example"              => Some(ElementType::Example),
                _                      => None,
            }
        })
    }
}
