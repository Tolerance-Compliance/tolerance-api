//! Application state for CMMC data
//!
//! The state is built once at startup and shared across all requests via Arc.
//! Supports multiple CMMC levels (L2 = SP 800-171, L3 = SP 800-172).

use std::collections::HashMap;
use std::sync::Arc;

use super::index::SearchIndex;
use super::model::{CmmcLevel, Element, NistData};

/// Data and index for a single CMMC level
struct LevelData {
    data: NistData,
    index: SearchIndex,
}

/// Shared state holding all loaded CMMC levels
#[derive(Clone)]
pub struct CmmcState {
    levels: Arc<HashMap<CmmcLevel, LevelData>>,
}

impl CmmcState {
    /// Create state from a set of (level, data) pairs
    pub fn new(datasets: Vec<(CmmcLevel, NistData)>) -> Self {
        let mut levels = HashMap::with_capacity(datasets.len());
        for (level, data) in datasets {
            let index = SearchIndex::build(&data.response.elements.elements);
            levels.insert(level, LevelData { data, index });
        }
        Self {
            levels: Arc::new(levels),
        }
    }

    /// Load a single CMMC level from a JSON file
    pub fn load_json(path: &str) -> Result<NistData, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents)
            .map_err(|e: serde_json::Error| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Get the available levels
    pub fn available_levels(&self) -> Vec<CmmcLevel> {
        self.levels.keys().copied().collect()
    }

    fn level_data(&self, level: CmmcLevel) -> Option<&LevelData> {
        self.levels.get(&level)
    }

    /// Get reference to the raw NIST data for a level
    pub fn data(&self, level: CmmcLevel) -> Option<&NistData> {
        self.level_data(level).map(|ld| &ld.data)
    }

    /// Get reference to the search index for a level
    pub fn index(&self, level: CmmcLevel) -> Option<&SearchIndex> {
        self.level_data(level).map(|ld| &ld.index)
    }

    /// Get reference to elements slice for a level
    pub fn elements(&self, level: CmmcLevel) -> Option<&[Element]> {
        self.level_data(level)
            .map(|ld| ld.data.response.elements.elements.as_slice())
    }

    /// Get element by index for a level
    pub fn get_element(&self, level: CmmcLevel, idx: usize) -> Option<&Element> {
        self.elements(level).and_then(|els| els.get(idx))
    }
}
