//! Application state for NIST data
//!
//! The state is built once at startup and shared across all requests via Arc.
//! Supports multiple NIST documents and revisions (SP 800-171 Rev 1/2/3, SP 800-172 v1).

use std::collections::HashMap;
use std::sync::Arc;

use super::index::SearchIndex;
use super::model::{CmmcLevel, Element, NistData, NistDocument, NistDocumentKey, NistRevision};

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

    /// Create state from legacy CMMC level pairs (deprecated)
    #[allow(deprecated)]
    pub fn from_levels(datasets: Vec<(CmmcLevel, NistData)>) -> Self {
        let converted: Vec<(NistDocumentKey, NistData)> = datasets
            .into_iter()
            .map(|(level, data)| {
                let key = match level {
                    CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
                    CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
                };
                (key, data)
            })
            .collect();
        Self::new(converted)
    }

    /// Load a single document from a JSON file
    pub fn load_json(path: &str) -> Result<NistData, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e: serde_json::Error| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Get all available document keys
    pub fn available_documents(&self) -> Vec<NistDocumentKey> {
        self.documents.keys().copied().collect()
    }

    /// Get the available levels (deprecated - use available_documents)
    #[allow(deprecated)]
    pub fn available_levels(&self) -> Vec<CmmcLevel> {
        self.documents
            .keys()
            .filter_map(|key| match (key.document, key.revision) {
                (NistDocument::Sp800171, NistRevision::Rev3) => Some(CmmcLevel::L2),
                (NistDocument::Sp800172, NistRevision::V1) => Some(CmmcLevel::L3),
                _ => None,
            })
            .collect()
    }

    fn document_data(&self, key: NistDocumentKey) -> Option<&DocumentData> {
        self.documents.get(&key)
    }

    /// Get reference to the raw NIST data for a document
    pub fn data(&self, key: NistDocumentKey) -> Option<&NistData> {
        self.document_data(key).map(|dd| &dd.data)
    }

    /// Get reference to the search index for a document
    pub fn index(&self, key: NistDocumentKey) -> Option<&SearchIndex> {
        self.document_data(key).map(|dd| &dd.index)
    }

    /// Get reference to elements slice for a document
    pub fn elements(&self, key: NistDocumentKey) -> Option<&[Element]> {
        self.document_data(key)
            .map(|dd| dd.data.response.elements.elements.as_slice())
    }

    /// Get element by index for a document
    pub fn get_element(&self, key: NistDocumentKey, idx: usize) -> Option<&Element> {
        self.elements(key).and_then(|els| els.get(idx))
    }

    // Deprecated methods for backward compatibility
    #[allow(deprecated)]
    fn level_data(&self, level: CmmcLevel) -> Option<&DocumentData> {
        let key = match level {
            CmmcLevel::L2 => NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3),
            CmmcLevel::L3 => NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1),
        };
        self.document_data(key)
    }

    /// Get reference to the raw NIST data for a level (deprecated)
    #[allow(deprecated)]
    #[deprecated(note = "Use data(NistDocumentKey) instead")]
    pub fn data_by_level(&self, level: CmmcLevel) -> Option<&NistData> {
        self.level_data(level).map(|ld| &ld.data)
    }

    /// Get reference to the search index for a level (deprecated)
    #[allow(deprecated)]
    #[deprecated(note = "Use index(NistDocumentKey) instead")]
    pub fn index_by_level(&self, level: CmmcLevel) -> Option<&SearchIndex> {
        self.level_data(level).map(|ld| &ld.index)
    }

    /// Get reference to elements slice for a level (deprecated)
    #[allow(deprecated)]
    #[deprecated(note = "Use elements(NistDocumentKey) instead")]
    pub fn elements_by_level(&self, level: CmmcLevel) -> Option<&[Element]> {
        self.level_data(level)
            .map(|ld| ld.data.response.elements.elements.as_slice())
    }

    /// Get element by index for a level (deprecated)
    #[allow(deprecated)]
    #[deprecated(note = "Use get_element(NistDocumentKey, usize) instead")]
    pub fn get_element_by_level(&self, level: CmmcLevel, idx: usize) -> Option<&Element> {
        self.elements_by_level(level).and_then(|els| els.get(idx))
    }
}
