//! Framework-agnostic error type for the API logic.
//!
//! The consuming crates translate [`CoreError`] into their own transport error
//! (axum `IntoResponse` on native, a `worker::Response` in the Worker). The
//! status codes and JSON body shapes produced by [`CoreError::status`] and
//! [`CoreError::to_json`] are the contract both deployments must honor.

use serde_json::{json, Value};
use thiserror::Error;

/// API-specific errors for CMMC endpoints.
///
/// Mirrors the variants of the native `ApiError` so the HTTP responses are
/// identical regardless of which deployment served the request.
#[derive(Error, Debug, Clone)]
pub enum CoreError {
    /// 404 — the requested resource (document, family, element) was not found.
    #[error("Not found: {0}")]
    NotFound(String),
    /// 400 — the request could not be parsed (bad document/revision/etc.).
    #[error("Bad request: {0}")]
    BadRequest(String),
    /// 501 — the endpoint exists but does not support this document type.
    /// `hint` is a JSON object of suggested alternative endpoints.
    #[error("Not implemented: {message}")]
    NotImplemented { message: String, hint: Value },
}

impl CoreError {
    /// HTTP status code for this error.
    pub fn status(&self) -> u16 {
        match self {
            CoreError::NotFound(_) => 404,
            CoreError::BadRequest(_) => 400,
            CoreError::NotImplemented { .. } => 501,
        }
    }

    /// JSON body for this error, matching the API's `{ error, success }` shape
    /// (with an extra `hint` for `NotImplemented`).
    pub fn to_json(&self) -> Value {
        match self {
            CoreError::NotFound(message) | CoreError::BadRequest(message) => json!({
                "error": message,
                "success": false,
            }),
            CoreError::NotImplemented { message, hint } => json!({
                "error": message,
                "hint": hint,
                "success": false,
            }),
        }
    }
}
