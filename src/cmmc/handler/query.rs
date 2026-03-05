//! Query parameter types and path-parsing helpers

use serde::Deserialize;

use crate::cmmc::model::{ElementType, NistDocument, FarDocument, DocumentKey, DocumentRevision};
use crate::handler::error::ApiError;

/// Parse NIST document and revision path segments into a `DocumentKey`
pub fn parse_nist_document_key(document: &str, revision: &str) -> Result<DocumentKey, ApiError> {
    let doc = document
        .parse::<NistDocument>()
        .map_err(|e| ApiError::BadRequest(e))?;
    let rev = revision
        .parse::<DocumentRevision>()
        .map_err(|e| ApiError::BadRequest(e))?;

    match (doc, rev) {
        (NistDocument::Sp800171, DocumentRevision::V1 | DocumentRevision::V2) => {
            return Err(ApiError::BadRequest(
                "SP 800-171 uses revisions, not versions. Use r2 or r3 (e.g. /sp800-171/r3).".to_string(),
            ));
        }
        (NistDocument::Sp800172, DocumentRevision::Rev2 | DocumentRevision::Rev3) => {
            return Err(ApiError::BadRequest(
                "SP 800-172 uses versions, not revisions. Use v1 (e.g. /sp800-172/v1).".to_string(),
            ));
        }
        (NistDocument::Sp800172A, DocumentRevision::Rev2 | DocumentRevision::Rev3) => {
            return Err(ApiError::BadRequest(
                "SP 800-172A uses versions, not revisions. Use v1 (e.g. /sp800-172a/v1).".to_string(),
            ));
        }
        _ => {}
    }

    Ok(DocumentKey::nist(doc, rev))
}

/// Parse FAR document and revision path segments into a `DocumentKey`
pub fn parse_far_document_key(document: &str, revision: &str) -> Result<DocumentKey, ApiError> {
    let doc = document
        .parse::<FarDocument>()
        .map_err(|e| ApiError::BadRequest(e))?;
    let rev = revision
        .parse::<DocumentRevision>()
        .map_err(|e| ApiError::BadRequest(e))?;

    match (doc, rev) {
        (FarDocument::Far52_204_21, DocumentRevision::Rev2 | DocumentRevision::Rev3 | DocumentRevision::V1) => {
            return Err(ApiError::BadRequest(
                "FAR 52.204-21 uses v2 (e.g. /far/52.204-21/v2).".to_string(),
            ));
        }
        _ => {}
    }

    Ok(DocumentKey::far(doc, rev))
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
