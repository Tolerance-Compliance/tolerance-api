//! A single loaded document: its parsed data plus the pre-built search index.
//!
//! [`DocumentData`] is the owned, cacheable unit. The native service stores it
//! eagerly in a map at startup; the Worker builds it lazily on the first request
//! for a document and caches it per isolate. Either way, handlers borrow a
//! [`DocumentContext`] from it and pass that to the `service` functions.

use crate::index::SearchIndex;
use crate::model::{Document, Element, NistData, Relationship};

/// Parsed data and search index for a single document+revision.
pub struct DocumentData {
    data: NistData,
    index: SearchIndex,
}

impl DocumentData {
    /// Build from parsed NIST data, computing the search index.
    pub fn new(data: NistData) -> Self {
        let index = SearchIndex::build(&data.response.elements.elements);
        Self { data, index }
    }

    /// Borrow a combined view for the `service` functions.
    pub fn context(&self) -> DocumentContext<'_> {
        DocumentContext {
            elements: &self.data.response.elements.elements,
            relationships: &self.data.response.elements.relationships,
            index: &self.index,
            documents: &self.data.response.elements.documents,
        }
    }
}

/// Borrowed view over a single loaded document.
///
/// Bundles the four things every handler needs so callers make one lookup
/// instead of reaching into `NistData` repeatedly.
pub struct DocumentContext<'a> {
    /// All elements in the document.
    pub elements: &'a [Element],
    /// All relationships in the document.
    pub relationships: &'a [Relationship],
    /// Pre-built search index.
    pub index: &'a SearchIndex,
    /// Document metadata (name, version, website, etc.).
    pub documents: &'a [Document],
}
