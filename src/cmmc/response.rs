use serde::Serialize;
use utoipa::ToSchema;

use super::model::Document;
use super::scoring::RequirementScore;
use super::poam::PoamValidation;

/// A family with its nested requirements (API response)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Family {
    pub identifier: String,
    pub title: String,
    pub requirements: Vec<Requirement>,
}

/// A requirement with its security requirements (API response)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Requirement {
    pub identifier: String,
    pub title: String,
    pub text: String,
    pub security_requirements: Vec<SecurityRequirement>,
    /// CMMC scoring information (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<RequirementScore>,
    /// POA&M eligibility validation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poam_validation: Option<PoamValidation>,
}

/// A security requirement with discussion and assessment (API response)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SecurityRequirement {
    pub identifier: String,
    pub title: String,
    pub text: String,
    pub discussion: Option<String>,
    pub assessment: Option<String>,
    /// CMMC scoring information (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<RequirementScore>,
    /// POA&M eligibility validation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poam_validation: Option<PoamValidation>,
}

/// Summary statistics for the dataset
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DataSummary {
    pub document: Document,
    pub family_count: usize,
    pub requirement_count: usize,
    pub security_requirement_count: usize,
    pub relationship_count: usize,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data:     Vec<T>,
    pub total:    usize,
    pub limit:    usize,
    pub offset:   usize,
    pub has_more: bool,
}

// type aliases for OpenAPI documentation.
pub type         ElementListResponse = PaginatedResponse<crate::cmmc::model::Element>;
pub type          FamilyListResponse = Vec<Family>;
pub type     RequirementListResponse = Vec<crate::cmmc::model::Element>;
pub type    RelationshipListResponse = Vec<crate::cmmc::model::Relationship>;
pub type              FamilyResponse = Family;
pub type             ElementResponse = crate::cmmc::model::Element;
pub type ElementRelationshipResponse = Vec<crate::cmmc::model::Relationship>;
pub type             SummaryResponse = DataSummary;
