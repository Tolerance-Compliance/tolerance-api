//! Pure KV key builders and shared paging types.
//!
//! These are used by BOTH the offline ingest binary (native target) and the
//! Worker runtime (wasm target), so this module must not depend on `worker`,
//! `axum`, or any wasm-only crate.

use serde::{Deserialize, Serialize};

use crate::cmmc::model::DocumentKey;

/// Bumped whenever the ingest output format changes or data is re-published.
/// Keys are immutable per version, so a long `cacheTtl` is safe.
pub const KV_VERSION: &str = "1";

/// Fixed number of elements per stored page. The manifest records this so the
/// Worker can map an arbitrary `offset/limit` onto page numbers without reading
/// any page data.
pub const ELEMENTS_PER_PAGE: usize = 100;

/// Upper bound on how many search results we hydrate (and therefore how many
/// `el:{id}` reads a single search request can fan out to).
pub const SEARCH_PAGE_SIZE: usize = 50;

/// Cache TTL applied to every KV read. Keys are immutable per `KV_VERSION`.
pub const KV_CACHE_TTL_SECS: u64 = 86_400;

/// Table-of-contents for a paged collection. Lets the Worker compute which
/// page(s) a request needs without reading any data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageManifest {
    pub total: usize,
    pub page_size: usize,
    pub page_count: usize,
}

fn doc_prefix(key: DocumentKey) -> String {
    format!(
        "t:{}:{}:{}",
        KV_VERSION,
        key.document_string(),
        key.revision_string()
    )
}

/// `t:{v}:index` -> list of available documents.
pub fn index() -> String {
    format!("t:{}:index", KV_VERSION)
}

/// `t:{v}:{doc}:{rev}:summary`
pub fn summary(key: DocumentKey) -> String {
    format!("{}:summary", doc_prefix(key))
}

/// `t:{v}:{doc}:{rev}:families`
pub fn families(key: DocumentKey) -> String {
    format!("{}:families", doc_prefix(key))
}

/// `t:{v}:{doc}:{rev}:family:{id}`
pub fn family(key: DocumentKey, id: &str) -> String {
    format!("{}:family:{}", doc_prefix(key), id)
}

/// `t:{v}:{doc}:{rev}:el:{id}`
pub fn element(key: DocumentKey, id: &str) -> String {
    format!("{}:el:{}", doc_prefix(key), id)
}

/// `t:{v}:{doc}:{rev}:el:{id}:rels`
pub fn element_rels(key: DocumentKey, id: &str) -> String {
    format!("{}:el:{}:rels", doc_prefix(key), id)
}

/// `t:{v}:{doc}:{rev}:requirements`
pub fn requirements(key: DocumentKey) -> String {
    format!("{}:requirements", doc_prefix(key))
}

/// `t:{v}:{doc}:{rev}:secreq`
pub fn secreq(key: DocumentKey) -> String {
    format!("{}:secreq", doc_prefix(key))
}

/// `t:{v}:{doc}:{rev}:rels`
pub fn relationships(key: DocumentKey) -> String {
    format!("{}:rels", doc_prefix(key))
}

/// `t:{v}:{doc}:{rev}:elements:{bucket}:manifest`
pub fn elements_manifest(key: DocumentKey, bucket: &str) -> String {
    format!("{}:elements:{}:manifest", doc_prefix(key), bucket)
}

/// `t:{v}:{doc}:{rev}:elements:{bucket}:p{n}`
pub fn elements_page(key: DocumentKey, bucket: &str, n: usize) -> String {
    format!("{}:elements:{}:p{}", doc_prefix(key), bucket, n)
}

/// `t:{v}:{doc}:{rev}:ids:{bucket}` -> Vec<String> of element ids of that type.
/// Used to apply a `?type=` filter during search without hydrating bodies.
pub fn ids(key: DocumentKey, bucket: &str) -> String {
    format!("{}:ids:{}", doc_prefix(key), bucket)
}

/// `t:{v}:{doc}:{rev}:idx:tok:{token}` -> Vec<String> posting list of element ids.
pub fn token(key: DocumentKey, tok: &str) -> String {
    format!("{}:idx:tok:{}", doc_prefix(key), tok)
}
