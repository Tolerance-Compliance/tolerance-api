//! HTTP request handlers for CMMC API
//!
//! Handlers are organized by resource type:
//! - `nist` - New NIST API with document+revision structure (recommended)
//! - `summary` - Dataset summary statistics (legacy CMMC API)
//! - `families` - Security control families (legacy CMMC API)
//! - `elements` - Individual elements with search/filter (legacy CMMC API)
//! - `relationships` - Element relationships (legacy CMMC API)

pub mod query;
pub mod nist;
pub mod summary;
pub mod families;
pub mod elements;
pub mod relationships;

// New NIST API handlers (recommended)
pub use nist::{
    get_documents,
    get_summary as get_nist_summary,
    get_families as get_nist_families,
    get_family as get_nist_family,
    get_elements as get_nist_elements,
    get_element as get_nist_element,
    get_requirements as get_nist_requirements,
    get_security_requirements as get_nist_security_requirements,
    get_relationships as get_nist_relationships,
    get_element_relationships as get_nist_element_relationships,
};

// Legacy CMMC API handlers (deprecated)
pub use summary::get_summary;
pub use families::{get_families, get_family};
pub use elements::{get_elements, get_element, get_requirements, get_security_requirements};
pub use relationships::{get_relationships, get_element_relationships};
pub use query::ElementQuery;
