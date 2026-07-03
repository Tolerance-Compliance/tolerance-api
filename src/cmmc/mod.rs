//! CMMC / NIST 800-171 API module

pub mod format_response;
pub mod handler;
pub mod index;
pub mod model;
pub mod poam;
pub mod response;
pub mod scoring;
pub mod state;

// Re-export commonly used types
pub use index::SearchIndex;
pub use model::{
    DocumentKey,
    DocumentRevision,
    DocumentSource,
    Element,
    ElementType,
    FarDocument,
    NistData,
    NistDocument,
    NistDocumentKey,
    // Legacy aliases
    NistRevision,
    Relationship,
};
pub use poam::{PoamEligibility, PoamValidation, PoamValidator};
pub use response::{DataSummary, Family, PaginatedResponse, Requirement, SecurityRequirement};
pub use scoring::{CmmcLevel, Priority, RequirementScore, ScoringDatabase};
pub use state::CmmcState;
