//! Worker-side KV access layer (wasm target only).
//!
//! Wraps a `worker::kv::KvStore` in `SendWrapper` so it can be attached as
//! `axum` router state (axum requires `Send + Sync`, Worker JS handles are not).

use serde::de::DeserializeOwned;
use worker::kv::KvStore;
use worker::send::SendWrapper;

use crate::handler::error::ApiError;
use crate::kv::keys::KV_CACHE_TTL_SECS;

/// Cloneable, `Send`-able handle to the `DOCS` KV namespace, used as axum state.
#[derive(Clone)]
pub struct AppState {
    kv: SendWrapper<KvStore>,
}

impl AppState {
    pub fn new(kv: KvStore) -> Self {
        Self {
            kv: SendWrapper::new(kv),
        }
    }

    /// Read and deserialize a JSON value. Returns `Ok(None)` when the key is absent.
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        self.kv
            .get(key)
            .cache_ttl(KV_CACHE_TTL_SECS)
            .json::<T>()
            .await
            .map_err(|e| ApiError::Internal(format!("KV read failed for '{}': {:?}", key, e)))
    }
}
