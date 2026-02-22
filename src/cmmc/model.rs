//! Core data models for NIST SP 800-171 JSON structure
//!
//! These types map directly to the structure of the NIST CPRT JSON export.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Root structure for the NIST SP 800-171 JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistData {
    pub response: NistResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NistResponse {
    pub request_type: i32,
    pub elements: NistElements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistElements {
    pub documents: Vec<Document>,
    pub relationship_types: Vec<RelationshipType>,
    pub elements: Vec<Element>,
    pub relationships: Vec<Relationship>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Document {
    pub doc_identifier: String,
    pub name: String,
    pub version: String,
    pub website: String,
}

/// Relationship type definitions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RelationshipType {
    pub relationship_identifier: String,
    pub description: String,
}

/// CMMC maturity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum CmmcLevel {
    #[serde(rename = "l2")]
    L2,
    #[serde(rename = "l3")]
    L3,
}

impl CmmcLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            CmmcLevel::L2 => "l2",
            CmmcLevel::L3 => "l3",
        }
    }
}

impl std::fmt::Display for CmmcLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for CmmcLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "l2" | "2" => Ok(CmmcLevel::L2),
            "l3" | "3" => Ok(CmmcLevel::L3),
            _ => Err(format!("Unknown CMMC level: '{}'. Use 'l2' or 'l3'.", s)),
        }
    }
}

/// Element types in the NIST data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    Family,
    Requirement,
    SecurityRequirement,
    Discussion,
    Assessment,
    AdversaryEffect,
    ProtectionStrategy,
    Effect,
    Tactic,
    Impact,
    ExpectedResult,
    Example,
    Sort,
    ReferenceItem,
    #[serde(other)]
    Unknown,
}

/// A single element (family, requirement, or security requirement)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Element {
    pub element_type: ElementType,
    pub element_identifier: String,
    pub title: String,
    pub text: String,
    pub doc_identifier: String,
}

/// Relationship between elements
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Relationship {
    pub source_element_identifier: String,
    pub source_doc_identifier: String,
    pub dest_element_identifier: String,
    pub dest_doc_identifier: String,
    pub relationship_identifier: String,
    pub provenance_doc_identifier: String,
}
