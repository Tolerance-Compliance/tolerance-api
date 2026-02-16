//! HTTP request handlers for CMMC API
//!
//! Handlers are organized by resource type:
//! - `summary` - Dataset summary statistics
//! - `families` - Security control families
//! - `elements` - Individual elements with search/filter
//! - `relationships` - Element relationships

mod query;
mod summary;
mod families;
mod elements;
mod relationships;

// Re-export all handlers
pub use summary::get_summary;
pub use families::{get_families, get_family};
pub use elements::{get_elements, get_element, get_requirements, get_security_requirements};
pub use relationships::{get_relationships, get_element_relationships};

// Re-export query types for routing
pub use query::ElementQuery;
