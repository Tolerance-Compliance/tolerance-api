//! Catalog loading: lazy R2 fetch + per-isolate cache.
//!
//! The first request for a given document fetches its JSON from the `CATALOGS`
//! R2 bucket, parses it, builds the search index, and stashes the resulting
//! [`DocumentData`] in a per-isolate cache. Warm requests in the same isolate
//! reuse the cached `Arc` and skip R2 entirely.
//!
//! SP 800-53 (53/53A/53B) is intentionally not mapped: those large documents are
//! served only by the native Fly service. Requests for them resolve to a
//! `NotFound` here, matching "document not loaded".

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use worker::Env;

use tolerance_api_core::model::{DocumentKey, DocumentRevision, FarDocument, NistDocument, NistData};
use tolerance_api_core::DocumentData;

use crate::error::ApiError;

/// R2 binding name declared in `wrangler.toml`.
const CATALOGS_BUCKET: &str = "CATALOGS";

/// Per-isolate document cache. wasm isolates are single-threaded, but `Mutex`
/// keeps the borrow checker happy and the access pattern explicit.
static CACHE: OnceLock<Mutex<HashMap<DocumentKey, Arc<DocumentData>>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<DocumentKey, Arc<DocumentData>>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// R2 object name (under the `catalogs/` prefix) for a supported document.
///
/// Returns `None` for documents this Worker does not serve (notably SP 800-53).
pub fn object_name(key: DocumentKey) -> Option<&'static str> {
    use DocumentRevision::*;
    match key {
        DocumentKey::Nist { document: NistDocument::Sp800171, revision: Rev1 } => {
            Some("catalogs/cprt-sp_800_171_1_0_0.json")
        }
        DocumentKey::Nist { document: NistDocument::Sp800171, revision: Rev2 } => {
            Some("catalogs/cprt-sp_800_171_2_0_0.json")
        }
        DocumentKey::Nist { document: NistDocument::Sp800171, revision: Rev3 } => {
            Some("catalogs/cprt-sp_800_171_3_0_0-20260215-171034.json")
        }
        DocumentKey::Nist { document: NistDocument::Sp800171A, revision: V1 } => {
            Some("catalogs/cprt-sp_800_171a_1_0_0.json")
        }
        DocumentKey::Nist { document: NistDocument::Sp800171A, revision: Rev3 } => {
            Some("catalogs/cprt-sp_800_171_a_3_0_0.json")
        }
        DocumentKey::Nist { document: NistDocument::Sp800172, revision: V1 } => {
            Some("catalogs/cprt-sp_800_172_1_0_0.json")
        }
        DocumentKey::Nist { document: NistDocument::Sp800172A, revision: V1 } => {
            Some("catalogs/cprt-sp_800_172a_1_0_0.json")
        }
        DocumentKey::Far { document: FarDocument::Far52_204_21, revision: V2 } => {
            Some("catalogs/cprt-far_52_204_21-20260305.json")
        }
        _ => None,
    }
}

/// The NIST documents this Worker serves (for `/v1/nist/documents`). FAR is
/// listed via its own surface, so it is excluded here.
pub fn supported_nist_keys() -> Vec<DocumentKey> {
    use DocumentRevision::*;
    vec![
        DocumentKey::nist(NistDocument::Sp800171, Rev1),
        DocumentKey::nist(NistDocument::Sp800171, Rev2),
        DocumentKey::nist(NistDocument::Sp800171, Rev3),
        DocumentKey::nist(NistDocument::Sp800171A, V1),
        DocumentKey::nist(NistDocument::Sp800171A, Rev3),
        DocumentKey::nist(NistDocument::Sp800172, V1),
        DocumentKey::nist(NistDocument::Sp800172A, V1),
    ]
}

/// Resolve a document, loading + indexing it from R2 on the first request and
/// caching it for the lifetime of the isolate.
pub async fn load(env: &Env, key: DocumentKey) -> Result<Arc<DocumentData>, ApiError> {
    if let Some(data) = cache().lock().unwrap().get(&key) {
        return Ok(data.clone());
    }

    let name = object_name(key)
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let object = env
        .bucket(CATALOGS_BUCKET)?
        .get(name)
        .execute()
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Document {} not loaded", key)))?;

    let body = object
        .body()
        .ok_or_else(|| ApiError::Internal(format!("catalog object {} has no body", name)))?;
    let bytes = body.bytes().await?;

    let data: NistData = serde_json::from_slice(&bytes)
        .map_err(|e| ApiError::Internal(format!("failed to parse {}: {}", name, e)))?;

    let arc = Arc::new(DocumentData::new(data));
    cache().lock().unwrap().insert(key, arc.clone());
    Ok(arc)
}
