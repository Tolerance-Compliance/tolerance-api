//! API response types
//!
//! These types are used for API responses and differ from the raw NIST data
//! by providing nested, hierarchical structures.

use serde::Serialize;

use super::model::Document;

/// A family with its nested requirements (API response)
#[derive(Debug, Clone, Serialize)]
pub struct Family {
    pub identifier: String,
    pub title: String,
    pub requirements: Vec<Requirement>,
}

/// A requirement with its security requirements (API response)
#[derive(Debug, Clone, Serialize)]
pub struct Requirement {
    pub identifier: String,
    pub title: String,
    pub text: String,
    pub security_requirements: Vec<SecurityRequirement>,
}

/// A security requirement with discussion and assessment (API response)
#[derive(Debug, Clone, Serialize)]
pub struct SecurityRequirement {
    pub identifier: String,
    pub title: String,
    pub text: String,
    pub discussion: Option<String>,
    pub assessment: Option<String>,
}

/// Summary statistics for the dataset
#[derive(Debug, Clone, Serialize)]
pub struct DataSummary {
    pub document: Document,
    pub family_count: usize,
    pub requirement_count: usize,
    pub security_requirement_count: usize,
    pub relationship_count: usize,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
}
