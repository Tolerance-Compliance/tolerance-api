//! Application state for CMMC data
//!
//! The state is built once at startup and shared across all requests via Arc.

use std::sync::Arc;

use super::index::SearchIndex;
use super::model::{Element, NistData};

/// Shared state for CMMC data with pre-built search index
#[derive(Clone)]
pub struct CmmcState {
    /// Raw NIST data
    data: Arc<NistData>,
    /// Pre-built search index for O(1) lookups
    index: Arc<SearchIndex>,
}

impl CmmcState {
    /// Create new state from NIST data, building the search index
    pub fn new(data: NistData) -> Self {
        let index: SearchIndex = SearchIndex::build(&data.response.elements.elements);
        Self {
            data: Arc::new(data),
            index: Arc::new(index),
        }
    }

    /// Load CMMC data from JSON file
    pub fn from_json_file(path: &str) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        let data: NistData = serde_json::from_str(&contents)
            .map_err(|e: serde_json::Error| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::new(data))
    }

    /// Get reference to the raw NIST data
    #[inline]
    pub fn data(&self) -> &NistData {
        &self.data
    }

    /// Get reference to the search index
    #[inline]
    pub fn index(&self) -> &SearchIndex {
        &self.index
    }

    /// Get reference to elements slice
    #[inline]
    pub fn elements(&self) -> &[Element] {
        &self.data.response.elements.elements
    }

    /// Get element by index
    #[inline]
    pub fn get_element(&self, idx: usize) -> Option<&Element> {
        self.data.response.elements.elements.get(idx)
    }
}
