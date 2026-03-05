//! CMMC / NIST 800-171 API module

pub mod model;
pub mod response;
pub mod index;
pub mod state;
pub mod handler;
pub mod format_response;
pub mod scoring;
pub mod poam;

// Re-export commonly used types
pub use state::CmmcState;
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
