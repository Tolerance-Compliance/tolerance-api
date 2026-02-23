//! Application state for NIST data
//!
//! The state is built once at startup and shared across all requests via Arc.
//! Supports multiple NIST documents and revisions (SP 800-171 Rev 1/2/3, SP 800-172 v1).

use std::collections::HashMap;
use std::sync::Arc;

use super::index::SearchIndex;
use super::model::{Document, Element, NistData, NistDocumentKey, Relationship};

/// Convenience view over a single loaded document.
///
/// Obtain via [`CmmcState::get_document`] to avoid separate calls to
/// `data()`, `index()`, and `elements()` — and the `.unwrap()` calls
/// that come with them.
pub struct DocumentContext<'a> {
    /// All elements in the document
    pub elements: &'a [Element],
    /// All relationships in the document
    pub relationships: &'a [Relationship],
    /// Pre-built search index
    pub index: &'a SearchIndex,
    /// Document metadata (name, version, website, etc.)
    pub documents: &'a [Document],
}

/// Data and index for a single document+revision
struct DocumentData {
    data: NistData,
    index: SearchIndex,
}

/// Shared state holding all loaded NIST documents
#[derive(Clone)]
pub struct CmmcState {
    documents: Arc<HashMap<NistDocumentKey, DocumentData>>,
}

impl CmmcState {
    /// Create state from a set of (key, data) pairs
    pub fn new(datasets: Vec<(NistDocumentKey, NistData)>) -> Self {
        let mut documents = HashMap::with_capacity(datasets.len());
        for (key, data) in datasets {
            let index = SearchIndex::build(&data.response.elements.elements);
            documents.insert(key, DocumentData { data, index });
        }
        Self {
            documents: Arc::new(documents),
        }
    }

    /// Load a single document from a JSON file
    pub fn load_json(path: &str) -> Result<NistData, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Get all available document keys
    pub fn available_documents(&self) -> Vec<NistDocumentKey> {
        self.documents.keys().copied().collect()
    }

    /// Get a combined view of elements, relationships, index, and metadata for a document.
    ///
    /// This is the preferred access method for handlers — one lookup instead of three.
    pub fn get_document(&self, key: NistDocumentKey) -> Option<DocumentContext<'_>> {
        self.documents.get(&key).map(|dd| DocumentContext {
            elements: &dd.data.response.elements.elements,
            relationships: &dd.data.response.elements.relationships,
            index: &dd.index,
            documents: &dd.data.response.elements.documents,
        })
    }

    /// Get element by index for a document
    pub fn get_element(&self, key: NistDocumentKey, idx: usize) -> Option<&Element> {
        self.documents
            .get(&key)
            .and_then(|dd| dd.data.response.elements.elements.get(idx))
    }
}
