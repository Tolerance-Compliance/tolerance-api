use std::collections::HashMap;
use std::sync::Arc;

use tolerance_api_core::document::{DocumentContext, DocumentData};
use tolerance_api_core::model::{DocumentKey, NistData};
use tolerance_api_core::poam::PoamValidator;
use tolerance_api_core::scoring::ScoringDatabase;

/// Shared state holding all loaded documents (NIST, FAR, etc.).
///
/// The native service loads every configured document from disk at startup and
/// keeps it in memory for the lifetime of the process. (The Worker resolves the
/// same `DocumentData` lazily from R2 instead — see `../worker`.)
#[derive(Clone)]
pub struct CmmcState {
    documents: Arc<HashMap<DocumentKey, DocumentData>>,
    scoring_db: Arc<ScoringDatabase>,
    poam_validator: Arc<PoamValidator>,
}

impl CmmcState {
    /// Create state from a set of (key, data) pairs.
    pub fn new(datasets: Vec<(DocumentKey, NistData)>) -> Self {
        let mut documents = HashMap::with_capacity(datasets.len());
        for (key, data) in datasets {
            documents.insert(key, DocumentData::new(data));
        }
        let scoring_db = Arc::new(ScoringDatabase::new());
        let poam_validator = Arc::new(PoamValidator::new(ScoringDatabase::new()));
        Self {
            documents: Arc::new(documents),
            scoring_db,
            poam_validator,
        }
    }

    /// Load a single document from a JSON file.
    pub fn load_json(path: &str) -> Result<NistData, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Get all available document keys.
    pub fn available_documents(&self) -> Vec<DocumentKey> {
        self.documents.keys().copied().collect()
    }

    /// Get a combined view of elements, relationships, index, and metadata for a document.
    pub fn get_document(&self, key: DocumentKey) -> Option<DocumentContext<'_>> {
        self.documents.get(&key).map(|dd| dd.context())
    }

    /// Get the scoring database.
    pub fn scoring_db(&self) -> &ScoringDatabase {
        &self.scoring_db
    }

    /// Get the POA&M validator.
    pub fn poam_validator(&self) -> &PoamValidator {
        &self.poam_validator
    }
}
