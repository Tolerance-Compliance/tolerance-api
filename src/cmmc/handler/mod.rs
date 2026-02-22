//! HTTP request handlers for CMMC API
//!
//! Handlers are organized by resource type:
//! - `summary` - Dataset summary statistics
//! - `families` - Security control families
//! - `elements` - Individual elements with search/filter
//! - `relationships` - Element relationships

pub mod query;
pub mod summary;
pub mod families;
pub mod elements;
pub mod relationships;

pub use summary::get_summary;
pub use families::{get_families, get_family};
pub use elements::{get_elements, get_element, get_requirements, get_security_requirements};
pub use relationships::{get_relationships, get_element_relationships};
pub use query::ElementQuery;
