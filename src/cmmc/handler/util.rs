//! Shared KV-read helpers for the NIST/POA&M handlers.

use serde::de::DeserializeOwned;

use crate::cmmc::model::{DocumentKey, Element};
use crate::cmmc::response::PaginatedResponse;
use crate::handler::error::ApiError;
use crate::kv::keys::{self, PageManifest};
use crate::kv::store::AppState;

/// Read a JSON value, returning `NotFound` (with `msg`) when the key is absent.
pub(super) async fn require<T: DeserializeOwned>(
    state: &AppState,
    key: &str,
    msg: String,
) -> Result<T, ApiError> {
    state
        .get_json::<T>(key)
        .await?
        .ok_or(ApiError::NotFound(msg))
}

/// Returns true when the document has been ingested (its `summary` key exists).
pub(super) async fn doc_loaded(state: &AppState, key: DocumentKey) -> Result<bool, ApiError> {
    Ok(state
        .get_json::<serde_json::Value>(&keys::summary(key))
        .await?
        .is_some())
}

/// Serve a paginated `elements` bucket from stored pages.
///
/// Reads the manifest, then only the page(s) covering `[offset, offset+limit)`.
/// When the manifest is missing but the document exists, returns an empty page
/// (valid type with zero matches); otherwise 404.
pub(super) async fn paginated_elements(
    state: &AppState,
    key: DocumentKey,
    bucket: &str,
    offset: usize,
    limit: usize,
) -> Result<PaginatedResponse<Element>, ApiError> {
    let manifest: Option<PageManifest> =
        state.get_json(&keys::elements_manifest(key, bucket)).await?;

    let manifest = match manifest {
        Some(m) => m,
        None => {
            if doc_loaded(state, key).await? {
                return Ok(PaginatedResponse {
                    data: Vec::new(),
                    total: 0,
                    limit,
                    offset,
                    has_more: false,
                });
            }
            return Err(ApiError::NotFound(format!("Document {} not loaded", key)));
        }
    };

    let data = read_span(state, key, bucket, &manifest, offset, limit).await?;
    Ok(PaginatedResponse {
        data,
        total: manifest.total,
        limit,
        offset,
        has_more: offset + limit < manifest.total,
    })
}

async fn read_span(
    state: &AppState,
    key: DocumentKey,
    bucket: &str,
    manifest: &PageManifest,
    offset: usize,
    limit: usize,
) -> Result<Vec<Element>, ApiError> {
    let page_size = manifest.page_size.max(1);
    let start = offset.min(manifest.total);
    let end = offset.saturating_add(limit).min(manifest.total);
    if start >= end {
        return Ok(Vec::new());
    }

    let first_page = start / page_size;
    let last_page = (end - 1) / page_size;

    let mut buf: Vec<Element> = Vec::new();
    for p in first_page..=last_page {
        let page: Option<Vec<Element>> =
            state.get_json(&keys::elements_page(key, bucket, p)).await?;
        match page {
            Some(items) => buf.extend(items),
            None => break,
        }
    }

    let local_start = start - first_page * page_size;
    let take = end - start;
    Ok(buf.into_iter().skip(local_start).take(take).collect())
}
