//! Core data models for NIST SP 800-171 JSON structure
//!
//! These types map directly to the structure of the NIST CPRT JSON export.

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_identifier: String,
    pub name: String,
    pub version: String,
    pub website: String,
}

/// Relationship type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipType {
    pub relationship_identifier: String,
    pub description: String,
}

/// Element types in the NIST data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    Family,
    Requirement,
    SecurityRequirement,
    Discussion,
    Assessment,
    #[serde(other)]
    Unknown,
}

/// A single element (family, requirement, or security requirement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub element_type: ElementType,
    pub element_identifier: String,
    pub title: String,
    pub text: String,
    pub doc_identifier: String,
}

/// Relationship between elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source_element_identifier: String,
    pub source_doc_identifier: String,
    pub dest_element_identifier: String,
    pub dest_doc_identifier: String,
    pub relationship_identifier: String,
    pub provenance_doc_identifier: String,
}
