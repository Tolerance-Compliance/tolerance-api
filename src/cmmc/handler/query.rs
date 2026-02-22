//! Query parameter types for CMMC API endpoints

use serde::Deserialize;

use crate::cmmc::model::{CmmcLevel, ElementType};
use crate::handler::error::ApiError;

/// Parse a level string from a path parameter into a CmmcLevel
pub fn parse_level(level: &str) -> Result<CmmcLevel, ApiError> {
    level
        .parse::<CmmcLevel>()
        .map_err(|e| ApiError::BadRequest(e))
}

/// Query parameters for filtering elements with pagination
#[derive(Debug, Deserialize)]
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
}
