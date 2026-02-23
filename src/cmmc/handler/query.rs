//! Query parameter types for CMMC API endpoints

use serde::Deserialize;

use crate::cmmc::model::{CmmcLevel, ElementType, NistDocument, NistDocumentKey, NistRevision};
use crate::handler::error::ApiError;

/// Query parameter for output format
///
/// Supports two formats:
/// - `json` (default): Standard JSON response
/// - `toon`: Token-Oriented Object Notation for LLM optimization
///
/// # TOON Format
///
/// TOON is a compact, human-readable format designed for LLM consumption:
/// - 30-40% token reduction compared to JSON
/// - Tabular arrays with explicit lengths: `[N]{fields}:`
/// - Indentation-based objects (similar to YAML)
/// - Smart quoting (only when necessary)
///
/// Recommended when passing API responses to LLMs for compliance guidance.
///
/// # Examples
///
/// JSON: `?format=json` or omit parameter
/// TOON: `?format=toon`
#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FormatQuery {
    /// Output format: 'json' (default) or 'toon'.
    ///
    /// TOON format is optimized for LLM consumption with 30-40% token reduction.
    /// Use TOON when passing responses to language models for compliance guidance.
    ///
    /// Example: `?format=toon`
    #[param(example = "json")]
    pub format: Option<String>,
}

impl FormatQuery {
    /// Check if TOON format is requested via query parameter
    pub fn is_toon(&self) -> bool {
        self.format.as_ref().map(|f| f.eq_ignore_ascii_case("toon")).unwrap_or(false)
    }

    /// Check if TOON format is requested via query parameter or Accept header
    pub fn is_toon_with_headers(&self, headers: &axum::http::HeaderMap) -> bool {
        // First check query parameter
        if self.is_toon() {
            return true;
        }

        // Then check Accept header
        if let Some(accept) = headers.get(axum::http::header::ACCEPT) {
            if let Ok(accept_str) = accept.to_str() {
                // Check if Accept header contains text/toon
                return accept_str.contains("text/toon");
            }
        }

        false
    }
}

/// Parse a document string from a path parameter into a NistDocument
pub fn parse_document(document: &str) -> Result<NistDocument, ApiError> {
    document
        .parse::<NistDocument>()
        .map_err(|e| ApiError::BadRequest(e))
}

/// Parse a revision string from a path parameter into a NistRevision
pub fn parse_revision(revision: &str) -> Result<NistRevision, ApiError> {
    revision
        .parse::<NistRevision>()
        .map_err(|e| ApiError::BadRequest(e))
}

/// Parse document and revision strings into a NistDocumentKey
pub fn parse_document_key(document: &str, revision: &str) -> Result<NistDocumentKey, ApiError> {
    let doc = parse_document(document)?;
    let rev = parse_revision(revision)?;

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

/// Parse a level string from a path parameter into a CmmcLevel (deprecated)
#[allow(deprecated)]
pub fn parse_level(level: &str) -> Result<CmmcLevel, ApiError> {
    level
        .parse::<CmmcLevel>()
        .map_err(|e| ApiError::BadRequest(e))
}

/// Query parameters for filtering elements with pagination
#[derive(Debug, Deserialize, utoipa::IntoParams)]
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
    /// Output format (json or toon, default: json)
    pub format: Option<String>,
}

impl ElementQuery {
    pub const DEFAULT_LIMIT: usize = 100;
    pub const MAX_LIMIT: usize = 1000;

    /// Get the limit, clamped to MAX_LIMIT
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(Self::DEFAULT_LIMIT).min(Self::MAX_LIMIT)
    }

    /// Get the offset
    pub fn offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }

    /// Parse the element type string into an ElementType
    pub fn parse_element_type(&self) -> Option<ElementType> {
        self.element_type.as_ref().and_then(|t| {
            match t.to_lowercase().as_str() {
                "family" => Some(ElementType::Family),
                "requirement" => Some(ElementType::Requirement),
                "security_requirement" => Some(ElementType::SecurityRequirement),
                "discussion" => Some(ElementType::Discussion),
                "assessment" => Some(ElementType::Assessment),
                "adversary_effect" => Some(ElementType::AdversaryEffect),
                "protection_strategy" => Some(ElementType::ProtectionStrategy),
                "effect" => Some(ElementType::Effect),
                "tactic" => Some(ElementType::Tactic),
                "impact" => Some(ElementType::Impact),
                "expected_result" => Some(ElementType::ExpectedResult),
                "example" => Some(ElementType::Example),
                _ => None,
            }
        })
    }

    /// Check if TOON format is requested
    pub fn is_toon(&self) -> bool {
        self.format.as_ref().map(|f| f.eq_ignore_ascii_case("toon")).unwrap_or(false)
    }
}
