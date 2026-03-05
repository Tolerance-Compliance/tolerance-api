use std::collections::HashMap;
use std::sync::Arc;

use super::index::SearchIndex;
use super::model::{Document, Element, NistData, DocumentKey, Relationship};
use super::scoring::ScoringDatabase;
use super::poam::PoamValidator;

/// Convenience view over a single loaded document.
///
/// Obtain via [`CmmcState::get_document`] to avoid separate calls to
/// `data()`, `index()`, and `elements()` — and the `.unwrap()` calls
/// that come with them.
pub struct DocumentContext<'a> {
    /// All elements in the document
    pub elements:      &'a [Element],
    /// All relationships in the document
    pub relationships: &'a [Relationship],
    /// Pre-built search index
    pub index:         &'a SearchIndex,
    /// Document metadata (name, version, website, etc.)
    pub documents:     &'a [Document],
}

/// Data and index for a single document+revision
struct DocumentData {
    data: NistData,
    index: SearchIndex,
}

/// Shared state holding all loaded documents (NIST, FAR, etc.)
#[derive(Clone)]
pub struct CmmcState {
    documents: Arc<HashMap<DocumentKey, DocumentData>>,
    scoring_db: Arc<ScoringDatabase>,
    poam_validator: Arc<PoamValidator>,
}

impl CmmcState {
    /// Create state from a set of (key, data) pairs
    pub fn new(datasets: Vec<(DocumentKey, NistData)>) -> Self {
        let mut documents = HashMap::with_capacity(datasets.len());
        for (key, data) in datasets {
            let index = SearchIndex::build(&data.response.elements.elements);
            documents.insert(key, DocumentData { data, index });
        }
        let scoring_db = Arc::new(ScoringDatabase::new());
        let poam_validator = Arc::new(PoamValidator::new(ScoringDatabase::new()));
        Self {
            documents: Arc::new(documents),
            scoring_db,
            poam_validator,
        }
    }

    /// Load a single document from a JSON file
    pub fn load_json(path: &str) -> Result<NistData, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Get all available document keys
    pub fn available_documents(&self) -> Vec<DocumentKey> {
        self.documents.keys().copied().collect()
    }

    /// Get a combined view of elements, relationships, index, and metadata for a document.
    ///
    /// This is the preferred access method for handlers — one lookup instead of three.
    pub fn get_document(&self, key: DocumentKey) -> Option<DocumentContext<'_>> {
        self.documents.get(&key).map(|dd| DocumentContext {
            elements: &dd.data.response.elements.elements,
            relationships: &dd.data.response.elements.relationships,
            index: &dd.index,
            documents: &dd.data.response.elements.documents,
        })
    }

    /// Get element by index for a document
    pub fn get_element(&self, key: DocumentKey, idx: usize) -> Option<&Element> {
        self.documents
            .get(&key)
            .and_then(|dd| dd.data.response.elements.elements.get(idx))
    }

    /// Get the scoring database
    pub fn scoring_db(&self) -> &ScoringDatabase {
        &self.scoring_db
    }

    /// Get the POA&M validator
    pub fn poam_validator(&self) -> &PoamValidator {
        &self.poam_validator
    }
}
