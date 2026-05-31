//! CMMC / NIST 800-171 API module (native axum service).
//!
//! The framework-agnostic data models, search index, scoring/POA&M logic,
//! request parsing, response shaping, and endpoint logic live in the
//! `tolerance-api-core` crate. They are re-exported here so existing
//! `crate::cmmc::*` paths keep resolving. This module only adds the native
//! transport layer: the eager in-memory [`state::CmmcState`], the axum
//! handlers, and the axum `FormatResponse`.

// Framework-agnostic pieces, re-exported from core under their original paths.
pub use tolerance_api_core::{index, model, poam, query, response, scoring, service};

// Native-only transport layer.
pub mod format_response;
pub mod handler;
pub mod state;

// Re-export commonly used types (matches the pre-workspace surface).
pub use state::CmmcState;
pub use tolerance_api_core::{
    DataSummary, DocumentKey, DocumentRevision, DocumentSource, Element, ElementType, FarDocument,
    Family, NistData, NistDocument, NistDocumentKey, NistRevision, PaginatedResponse,
    Relationship, Requirement, SearchIndex, SecurityRequirement,
};
pub use tolerance_api_core::{
    CmmcLevel, PoamEligibility, PoamValidation, PoamValidator, Priority, RequirementScore,
    ScoringDatabase,
};
