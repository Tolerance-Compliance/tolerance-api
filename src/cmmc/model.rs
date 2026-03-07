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

/// Document source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DocumentSource {
    Nist,
    Far,
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

/// FAR (Federal Acquisition Regulation) document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FarDocument {
    #[serde(rename = "52.204-21")]
    Far52_204_21,
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
                "Unknown NIST document: '{}'. Use 'sp800-171', 'sp800-171a', 'sp800-172', or 'sp800-172a'.", s
            )),
        }
    }
}

impl FarDocument {
    pub fn as_str(&self) -> &'static str {
        match self {
            FarDocument::Far52_204_21 => "52.204-21",
        }
    }
}

impl std::fmt::Display for FarDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for FarDocument {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "52.204-21" | "52.204.21" | "52-204-21" => Ok(FarDocument::Far52_204_21),
            _ => Err(format!(
                "Unknown FAR document: '{}'. Use '52.204-21'.", s
            )),
        }
    }
}

/// Document revision/version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum DocumentRevision {
    #[serde(rename = "r1")]
    Rev1,
    #[serde(rename = "r2")]
    Rev2,
    #[serde(rename = "r3")]
    Rev3,
    #[serde(rename = "v1")]
    V1,
    #[serde(rename = "v2")]
    V2,
}

/// Legacy alias for backward compatibility
pub type NistRevision = DocumentRevision;

impl DocumentRevision {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentRevision::Rev1 => "r1",
            DocumentRevision::Rev2 => "r2",
            DocumentRevision::Rev3 => "r3",
            DocumentRevision::V1   => "v1",
            DocumentRevision::V2   => "v2",
        }
    }
}

impl std::fmt::Display for DocumentRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for DocumentRevision {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "r1" | "rev1" | "1" => Ok(DocumentRevision::Rev1),
            "r2" | "rev2" | "2" => Ok(DocumentRevision::Rev2),
            "r3" | "rev3" | "3" => Ok(DocumentRevision::Rev3),
            "v1" | "1.0"        => Ok(DocumentRevision::V1),
            "v2" | "2.0"        => Ok(DocumentRevision::V2),
            _ => Err(format!("Unknown revision: '{}'. Use 'r1', 'r2', 'r3', 'v1', or 'v2'.", s)),
        }
    }
}

/// Unified document key for all document types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum DocumentKey {
    Nist { document: NistDocument, revision: DocumentRevision },
    Far { document: FarDocument, revision: DocumentRevision },
}

impl DocumentKey {
    pub fn nist(document: NistDocument, revision: DocumentRevision) -> Self {
        Self::Nist { document, revision }
    }

    pub fn far(document: FarDocument, revision: DocumentRevision) -> Self {
        Self::Far { document, revision }
    }

    /// The CPRT framework version identifier
    pub fn cprt_identifier(&self) -> &'static str {
        match self {
            DocumentKey::Nist { document, revision } => match (document, revision) {
                (NistDocument::Sp800171,  DocumentRevision::Rev1) => "sp_800_171_1_0_0",
                (NistDocument::Sp800171,  DocumentRevision::Rev2) => "sp_800_171_2_0_0",
                (NistDocument::Sp800171,  DocumentRevision::Rev3) => "sp_800_171_3_0_0",
                (NistDocument::Sp800171A, DocumentRevision::V1)   => "sp_800_171a_1_0_0",
                (NistDocument::Sp800171A, DocumentRevision::Rev3) => "sp_800_171_a_3_0_0",
                (NistDocument::Sp800172,  DocumentRevision::V1)   => "sp_800_172_1_0_0",
                (NistDocument::Sp800172A, DocumentRevision::V1)   => "sp_800_172a_1_0_0",
                _ => "unknown",
            },
            DocumentKey::Far { document, revision } => match (document, revision) {
                (FarDocument::Far52_204_21, DocumentRevision::V2) => "far_52_204_21",
                _ => "unknown",
            },
        }
    }

    /// Human-readable name for the document
    pub fn display_name(&self) -> String {
        match self {
            DocumentKey::Nist { document, revision } => match (document, revision) {
                (NistDocument::Sp800171,  DocumentRevision::Rev1) => "SP 800-171 Rev 1".to_string(),
                (NistDocument::Sp800171,  DocumentRevision::Rev2) => "SP 800-171 Rev 2".to_string(),
                (NistDocument::Sp800171,  DocumentRevision::Rev3) => "SP 800-171 Rev 3".to_string(),
                (NistDocument::Sp800171A, DocumentRevision::V1)   => "SP 800-171A v1.0".to_string(),
                (NistDocument::Sp800171A, DocumentRevision::Rev3) => "SP 800-171A Rev 3".to_string(),
                (NistDocument::Sp800172,  DocumentRevision::V1)   => "SP 800-172 v1.0".to_string(),
                (NistDocument::Sp800172A, DocumentRevision::V1)   => "SP 800-172A v1.0".to_string(),
                _ => format!("{} {}", document, revision),
            },
            DocumentKey::Far { document, revision } => match (document, revision) {
                (FarDocument::Far52_204_21, DocumentRevision::V2) => "FAR 52.204-21".to_string(),
                _ => format!("FAR {} {}", document, revision),
            },
        }
    }

    pub fn source(&self) -> DocumentSource {
        match self {
            DocumentKey::Nist { .. } => DocumentSource::Nist,
            DocumentKey::Far { .. } => DocumentSource::Far,
        }
    }

    pub fn document_string(&self) -> String {
        match self {
            DocumentKey::Nist { document, .. } => document.to_string(),
            DocumentKey::Far { document, .. } => document.to_string(),
        }
    }

    pub fn revision_string(&self) -> String {
        match self {
            DocumentKey::Nist { revision, .. } => revision.to_string(),
            DocumentKey::Far { revision, .. } => revision.to_string(),
        }
    }
}

impl std::fmt::Display for DocumentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentKey::Nist { document, revision } => write!(f, "{}/{}", document, revision),
            DocumentKey::Far { document, revision } => write!(f, "{}/{}", document, revision),
        }
    }
}

/// Legacy type alias for backward compatibility
pub type NistDocumentKey = DocumentKey;


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
