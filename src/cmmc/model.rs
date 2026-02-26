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
    pub request_type:       i32,
    pub elements:           NistElements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NistElements {
    pub documents:          Vec<Document>,
    pub relationship_types: Vec<RelationshipType>,
    pub elements:           Vec<Element>,
    pub relationships:      Vec<Relationship>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Document {
    pub doc_identifier:  String,
    pub name: String,
    pub version: String,
    pub website:                  String,
}

/// Relationship type definitions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RelationshipType {
    pub relationship_identifier: String,
    pub description:             String,
}

/// NIST document family
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NistDocument {
    #[serde(rename = "sp800-171")]
    Sp800171,
    #[serde(rename = "sp800-171a")]
    Sp800171A,
    #[serde(rename = "sp800-172")]
    Sp800172,
    #[serde(rename = "sp800-172a")]
    Sp800172A,
}

impl NistDocument {
    pub fn as_str(&self) -> &'static str {
        match self {
            NistDocument::Sp800171  => "sp800-171",
            NistDocument::Sp800171A => "sp800-171a",
            NistDocument::Sp800172  => "sp800-172",
            NistDocument::Sp800172A => "sp800-172a",
        }
    }
}

impl std::fmt::Display for NistDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for NistDocument {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sp800-171"  | "800-171"  | "171"  => Ok(NistDocument::Sp800171),
            "sp800-171a" | "800-171a" | "171a" => Ok(NistDocument::Sp800171A),
            "sp800-172"  | "800-172"  | "172"  => Ok(NistDocument::Sp800172),
            "sp800-172a" | "800-172a" | "172a" => Ok(NistDocument::Sp800172A),
            _ => Err(format!(
                "Unknown document: '{}'. Use 'sp800-171', 'sp800-171a', 'sp800-172', or 'sp800-172a'.", s
            )),
        }
    }
}

/// Revision of a NIST document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum NistRevision {
    #[serde(rename = "r2")]
    Rev2,
    #[serde(rename = "r3")]
    Rev3,
    /// For SP 800-172 which uses a version scheme (v1, v2, ...)
    #[serde(rename = "v1")]
    V1,
}

impl NistRevision {
    pub fn as_str(&self) -> &'static str {
        match self {
            NistRevision::Rev2 => "r2",
            NistRevision::Rev3 => "r3",
            NistRevision::V1   => "v1",
        }
    }
}

impl std::fmt::Display for NistRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for NistRevision {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "r2" | "rev2" | "2" => Ok(NistRevision::Rev2),
            "r3" | "rev3" | "3" => Ok(NistRevision::Rev3),
            "v1" | "1.0"        => Ok(NistRevision::V1),
            _ => Err(format!("Unknown revision: '{}'. Use 'r2', 'r3', or 'v1'.", s)),
        }
    }
}

/// Compound key identifying a specific document + revision in the state map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct NistDocumentKey {
    pub document: NistDocument,
    pub revision: NistRevision,
}

impl NistDocumentKey {
    pub fn new(document: NistDocument, revision: NistRevision) -> Self {
        Self { document, revision }
    }

    /// The CPRT framework version identifier used in the NIST API URL
    pub fn cprt_identifier(&self) -> &'static str {
        match (self.document, self.revision) {
            (NistDocument::Sp800171,  NistRevision::Rev2) => "sp_800_171_2_0_0",
            (NistDocument::Sp800171,  NistRevision::Rev3) => "sp_800_171_3_0_0",
            (NistDocument::Sp800171A, NistRevision::Rev3) => "sp_800_171_a_3_0_0",
            (NistDocument::Sp800172,  NistRevision::V1)   => "sp_800_172_1_0_0",
            (NistDocument::Sp800172A, NistRevision::V1)   => "sp_800_172a_1_0_0",
            _ => "unknown",
        }
    }

    /// Human-readable name for the document
    pub fn display_name(&self) -> String {
        match (self.document, self.revision) {
            (NistDocument::Sp800171,  NistRevision::Rev2) => "SP 800-171 Rev 2".to_string(),
            (NistDocument::Sp800171,  NistRevision::Rev3) => "SP 800-171 Rev 3".to_string(),
            (NistDocument::Sp800171A, NistRevision::Rev3) => "SP 800-171A Rev 3".to_string(),
            (NistDocument::Sp800172,  NistRevision::V1)   => "SP 800-172 v1.0".to_string(),
            (NistDocument::Sp800172A, NistRevision::V1)   => "SP 800-172A v1.0".to_string(),
            _ => format!("{} {}", self.document, self.revision),
        }
    }
}

impl std::fmt::Display for NistDocumentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.document, self.revision)
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
    Determination,
    Examine,
    Interview,
    Test,
    Odp,
    OdpStatement,
    OdpType,
    WithdrawReason,
    #[serde(other)]
    Unknown,
}

/// A single element (family, requirement, or security requirement)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Element {
    pub element_type:       ElementType,
    pub element_identifier: String,
    pub title:              String,
    pub text:               String,
    pub doc_identifier:     String,
}

/// Relationship between elements
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Relationship {
    pub source_element_identifier: String,
    pub source_doc_identifier:     String,
    pub dest_element_identifier:   String,
    pub dest_doc_identifier:       String,
    pub relationship_identifier:   String,
    pub provenance_doc_identifier: String,
}
