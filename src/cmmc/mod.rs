//! CMMC / NIST 800-171 API module
//!
//! This module provides a complete API for querying NIST SP 800-171 Rev 3
//! security requirements data.
//!
//! # Module Structure
//! - `model` - Core data structures from the NIST JSON
//! - `response` - API response types (families, requirements, pagination)
//! - `index` - Search index for O(1) lookups
//! - `state` - Application state management
//! - `handler` - HTTP request handlers

pub mod model;
pub mod response;
pub mod index;
pub mod state;
pub mod handler;

// Re-export commonly used types
pub use state::CmmcState;
pub use model::{NistData, Element, ElementType, Relationship, CmmcLevel};
pub use response::{Family, Requirement, SecurityRequirement, PaginatedResponse, DataSummary};
pub use index::SearchIndex;
