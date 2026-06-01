//! CMMC / NIST 800-171 API module

pub mod model;
pub mod response;
pub mod index;
pub mod assemble;
pub mod scoring;
pub mod poam;

// Worker-only modules (depend on `axum` extractors / `worker`).
#[cfg(target_arch = "wasm32")]
pub mod handler;
#[cfg(target_arch = "wasm32")]
pub mod format_response;

// Re-export commonly used types
pub use model::{
    NistData, Element, ElementType, Relationship,
    DocumentSource, NistDocument, FarDocument,
    DocumentRevision, DocumentKey,
    // Legacy aliases
    NistRevision, NistDocumentKey,
};
pub use response::{Family, Requirement, SecurityRequirement, PaginatedResponse, DataSummary};
pub use index::SearchIndex;
pub use scoring::{ScoringDatabase, RequirementScore, CmmcLevel, Priority};
pub use poam::{PoamValidator, PoamValidation, PoamEligibility};
