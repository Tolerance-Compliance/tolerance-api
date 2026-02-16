//! Summary endpoint handler

use axum::{extract::State, Json};

use crate::cmmc::model::{Document, ElementType};
use crate::cmmc::response::DataSummary;
use crate::cmmc::state::CmmcState;

/// Get summary of the NIST 800-171 data - O(1) using pre-computed counts
pub async fn get_summary(State(state): State<CmmcState>) -> Json<DataSummary> {
    let data = state.data();
    let index = state.index();

    let summary = DataSummary {
        document: data.response.elements.documents.first().cloned().unwrap_or_else(|| {
            Document {
                doc_identifier: String::new(),
                name: String::new(),
                version: String::new(),
                website: String::new(),
            }
        }),
        family_count: index.count_by_type(ElementType::Family),
        requirement_count: index.count_by_type(ElementType::Requirement),
        security_requirement_count: index.count_by_type(ElementType::SecurityRequirement),
        relationship_count: data.response.elements.relationships.len(),
    };

    Json(summary)
}
