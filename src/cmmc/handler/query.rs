//! Query parameter types and path-parsing helpers

use serde::Deserialize;

use crate::cmmc::model::{DocumentKey, DocumentRevision, ElementType, FarDocument, NistDocument};
use crate::handler::error::ApiError;

/// Returns `Err(NotImplemented)` if `key` is an SP 800-53 document.
///
/// The `/families`, `/requirements`, and `/security-requirements` endpoints rely on the
/// SP 800-171/172 hierarchical model (Family → Requirement → SecurityRequirement) and
/// identifier naming conventions (`03.01`, `SR-03.01.01`). SP 800-53 uses a flat control
/// model (`control`, `control_enhancement`) with different identifier patterns (`AC-1`,
/// `AC-1(1)`), so those endpoints return misleading empty results for 800-53 documents.
///
/// The error response includes a `hint` object with the correct replacement URLs so the
/// caller can immediately switch to the right endpoint.
pub fn require_cmmc_structured(key: DocumentKey) -> Result<(), ApiError> {
    let is_sp800_53 = matches!(
        key,
        DocumentKey::Nist {
            document: NistDocument::Sp800053 | NistDocument::Sp800053A | NistDocument::Sp800053B,
            ..
        }
    );

    if !is_sp800_53 {
        return Ok(());
    }

    let base = format!(
        "/v1/nist/{}/{}",
        key.document_string(),
        key.revision_string()
    );
    Err(ApiError::NotImplemented {
        message: format!(
            "The /families, /requirements, and /security-requirements endpoints use the \
             SP 800-171/172 hierarchical structure (Family → Requirement → SecurityRequirement) \
             and SP 800-171-style identifiers (e.g. 03.01.01, SR-03.01.01.a). \
             SP 800-53 uses a flat control model with identifiers like AC-1 and AC-1(1). \
             Use the /elements endpoint with a ?type filter instead."
        ),
        hint: serde_json::json!({
            "families":             format!("{}/elements?type=family", base),
            "controls":             format!("{}/elements?type=control", base),
            "control_enhancements": format!("{}/elements?type=control_enhancement", base),
            "search":               format!("{}/elements?search=<term>", base),
            "all_elements":         format!("{}/elements", base),
            "relationships":        format!("{}/relationships", base),
            "docs":                 "https://github.com/IronShield-Tech/tolerance-api#sp-800-53",
        }),
    })
}

/// Parse NIST document and revision path segments into a `DocumentKey`
pub fn parse_nist_document_key(document: &str, revision: &str) -> Result<DocumentKey, ApiError> {
    let doc = document
        .parse::<NistDocument>()
        .map_err(|e| ApiError::BadRequest(e))?;
    let rev = revision
        .parse::<DocumentRevision>()
        .map_err(|e| ApiError::BadRequest(e))?;

    match (doc, rev) {
        (
            NistDocument::Sp800053 | NistDocument::Sp800053A | NistDocument::Sp800053B,
            DocumentRevision::V1 | DocumentRevision::V2,
        ) => {
            return Err(ApiError::BadRequest(
                "SP 800-53 documents use revisions, not versions. Use r5 (e.g. /sp800-53/r5)."
                    .to_string(),
            ));
        }
        (NistDocument::Sp800171, DocumentRevision::V1 | DocumentRevision::V2) => {
            return Err(ApiError::BadRequest(
                "SP 800-171 uses revisions, not versions. Use r1, r2, or r3 (e.g. /sp800-171/r3)."
                    .to_string(),
            ));
        }
        (
            NistDocument::Sp800172,
            DocumentRevision::Rev1
            | DocumentRevision::Rev2
            | DocumentRevision::Rev3
            | DocumentRevision::Rev5,
        ) => {
            return Err(ApiError::BadRequest(
                "SP 800-172 uses versions, not revisions. Use v1 (e.g. /sp800-172/v1).".to_string(),
            ));
        }
        (
            NistDocument::Sp800172A,
            DocumentRevision::Rev1
            | DocumentRevision::Rev2
            | DocumentRevision::Rev3
            | DocumentRevision::Rev5,
        ) => {
            return Err(ApiError::BadRequest(
                "SP 800-172A uses versions, not revisions. Use v1 (e.g. /sp800-172a/v1)."
                    .to_string(),
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
        (
            FarDocument::Far52_204_21,
            DocumentRevision::Rev2 | DocumentRevision::Rev3 | DocumentRevision::V1,
        ) => {
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
    /// Maximum number of results (default: 100, max: 5000)
    pub limit: Option<usize>,
    /// Offset for pagination (default: 0)
    pub offset: Option<usize>,
}

impl ElementQuery {
    pub const DEFAULT_LIMIT: usize = 100;
    pub const MAX_LIMIT: usize = 5000;

    pub fn limit(&self) -> usize {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .min(Self::MAX_LIMIT)
    }

    pub fn offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }

    pub fn parse_element_type(&self) -> Option<ElementType> {
        self.element_type.as_ref().and_then(|t| {
            match t.to_lowercase().as_str() {
                // SP 800-171 / 800-172 types
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
                // Assessment-guide types (171A / 172A; also present in 800-53A)
                "determination" => Some(ElementType::Determination),
                "examine" => Some(ElementType::Examine),
                "interview" => Some(ElementType::Interview),
                "test" => Some(ElementType::Test),
                "odp" => Some(ElementType::Odp),
                "odp_statement" => Some(ElementType::OdpStatement),
                "odp_type" => Some(ElementType::OdpType),
                "sort" => Some(ElementType::Sort),
                "reference_item" => Some(ElementType::ReferenceItem),
                "withdraw_reason" => Some(ElementType::WithdrawReason),
                // SP 800-53 types
                "control" => Some(ElementType::Control),
                "control_enhancement" => Some(ElementType::ControlEnhancement),
                "control_statement" => Some(ElementType::ControlStatement),
                "control_name_sort" => Some(ElementType::ControlNameSort),
                "security_baseline" => Some(ElementType::SecurityBaseline),
                "privacy_baseline" => Some(ElementType::PrivacyBaseline),
                "reference" => Some(ElementType::Reference),
                "public_comment" => Some(ElementType::PublicComment),
                _ => None,
            }
        })
    }
}
