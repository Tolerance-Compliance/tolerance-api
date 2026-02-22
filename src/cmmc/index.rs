//! Search index for O(1) lookups and fast text search
//!
//! The search index is built once at application startup and provides:
//! - O(1) lookup by element identifier
//! - O(1) filtering by element type
//! - Fast text search via inverted index

use std::collections::{HashMap, HashSet};

use super::model::{Element, ElementType};

/// Pre-computed lowercase fields for fast search without repeated allocations
#[derive(Debug, Clone)]
pub struct IndexedElement {
    /// Index into the original elements array
    pub index: usize,
    /// Pre-computed lowercase identifier
    pub identifier_lower: String,
    /// Pre-computed lowercase title
    pub title_lower: String,
    /// Pre-computed lowercase text
    pub text_lower: String,
    /// Tokenized words from all searchable fields (for inverted index)
    pub tokens: HashSet<String>,
}

/// High-performance search index built at load time
#[derive(Debug, Clone)]
pub struct SearchIndex {
    /// Pre-indexed elements with lowercase fields
    pub indexed_elements: Vec<IndexedElement>,
    /// O(1) lookup by element identifier
    pub by_identifier: HashMap<String, usize>,
    /// Inverted index: word -> set of element indices
    pub inverted_index: HashMap<String, HashSet<usize>>,
    /// Elements grouped by type for fast type filtering
    pub by_type: HashMap<ElementType, Vec<usize>>,
    /// Pre-computed counts by type
    pub type_counts: HashMap<ElementType, usize>,
}

impl SearchIndex {
    /// Build the search index from raw elements - O(n * m) one-time cost
    pub fn build(elements: &[Element]) -> Self {
        let mut indexed_elements = Vec::with_capacity(elements.len());
        let mut by_identifier = HashMap::with_capacity(elements.len());
        let mut inverted_index: HashMap<String, HashSet<usize>> = HashMap::new();
        let mut by_type: HashMap<ElementType, Vec<usize>> = HashMap::new();
        let mut type_counts: HashMap<ElementType, usize> = HashMap::new();

        for (index, element) in elements.iter().enumerate() {
            let identifier_lower: String = element.element_identifier.to_lowercase();
            let title_lower: String = element.title.to_lowercase();
            let text_lower: String = element.text.to_lowercase();

            // Tokenize for inverted index
            let mut tokens: HashSet<String> = HashSet::new();
            for word in tokenize(&identifier_lower) {
                tokens.insert(word.clone());
                inverted_index.entry(word).or_default().insert(index);
            }
            for word in tokenize(&title_lower) {
                tokens.insert(word.clone());
                inverted_index.entry(word).or_default().insert(index);
            }
            for word in tokenize(&text_lower) {
                tokens.insert(word.clone());
                inverted_index.entry(word).or_default().insert(index);
            }

            // Build identifier lookup
            by_identifier.insert(element.element_identifier.clone(), index);

            // Group by type
            by_type.entry(element.element_type).or_default().push(index);
            *type_counts.entry(element.element_type).or_default() += 1;

            indexed_elements.push(IndexedElement {
                index,
                identifier_lower,
                title_lower,
                text_lower,
                tokens,
            });
        }

        Self {
            indexed_elements,
            by_identifier,
            inverted_index,
            by_type,
            type_counts,
        }
    }

    /// Fast lookup by identifier - O(1)
    pub fn get_by_identifier(&self, id: &str) -> Option<usize> {
        self.by_identifier.get(id).copied()
    }

    /// Get all indices for a given element type - O(1)
    pub fn get_by_type(&self, element_type: ElementType) -> &[usize] {
        self.by_type
            .get(&element_type)
            .map(|v: &Vec<usize>| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get count for a given element type - O(1)
    pub fn count_by_type(&self, element_type: ElementType) -> usize {
        *self.type_counts.get(&element_type).unwrap_or(&0)
    }

    /// Search using inverted index for whole-word matches, falls back to substring for partial
    pub fn search(&self, query: &str, type_filter: Option<ElementType>) -> Vec<usize> {
        let query_lower = query.to_lowercase();
        let query_tokens: Vec<String> = tokenize(&query_lower);

        // Start with type filter if provided
        let candidate_indices: HashSet<usize> = if let Some(et) = type_filter {
            self.get_by_type(et).iter().copied().collect()
        } else {
            (0..self.indexed_elements.len()).collect()
        };

        // If query is a single token, try inverted index first
        if query_tokens.len() == 1 {
            let token: &String = &query_tokens[0];

            // Exact word match via inverted index - O(1)
            if let Some(matches) = self.inverted_index.get(token) {
                let results: Vec<usize> = matches
                    .intersection(&candidate_indices)
                    .copied()
                    .collect();
                if !results.is_empty() {
                    return results;
                }
            }
        }

        // Fall back to substring search on pre-computed lowercase fields
        candidate_indices
            .into_iter()
            .filter(|&idx| {
                let indexed: &IndexedElement = &self.indexed_elements[idx];
                indexed.identifier_lower.contains(&query_lower)
                    || indexed.title_lower.contains(&query_lower)
                    || indexed.text_lower.contains(&query_lower)
            })
            .collect()
    }
}

/// Tokenize text into lowercase words (splits on non-alphanumeric)
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s: &&str| !s.is_empty() && s.len() > 1)
        .map(|s: &str| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_element(id: &str, title: &str, text: &str, element_type: ElementType) -> Element {
        Element {
            element_type,
            element_identifier: id.to_string(),
            title: title.to_string(),
            text: text.to_string(),
            doc_identifier: "TEST".to_string(),
        }
    }

    #[test]
    fn test_lookup_by_identifier() {
        let elements: Vec<Element> = vec![
            make_element("03.01", "Access Control", "", ElementType::Family),
            make_element("03.02", "Awareness", "", ElementType::Family),
        ];
        let index = SearchIndex::build(&elements);

        assert_eq!(index.get_by_identifier("03.01"), Some(0));
        assert_eq!(index.get_by_identifier("03.02"), Some(1));
        assert_eq!(index.get_by_identifier("03.03"), None);
    }

    #[test]
    fn test_filter_by_type() {
        let elements: Vec<Element> = vec![
            make_element("03.01", "Access Control", "", ElementType::Family),
            make_element("03.01.01", "Account Mgmt", "", ElementType::Requirement),
        ];
        let index = SearchIndex::build(&elements);

        assert_eq!(index.get_by_type(ElementType::Family).len(), 1);
        assert_eq!(index.get_by_type(ElementType::Requirement).len(), 1);
        assert_eq!(index.count_by_type(ElementType::Family), 1);
    }

    #[test]
    fn test_search() {
        let elements: Vec<Element> = vec![
            make_element("03.01", "Access Control", "manage access", ElementType::Family),
            make_element("03.02", "Training", "security training", ElementType::Family),
        ];
        let index: SearchIndex = SearchIndex::build(&elements);

        let results = index.search("access", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }
}
