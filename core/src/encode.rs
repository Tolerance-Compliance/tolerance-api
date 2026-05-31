//! Content-negotiation encoding shared by both deployments.
//!
//! Mirrors the native `FormatResponse`: a successful body is serialized to JSON
//! (`application/json`) by default, or TOON (`text/toon; charset=utf-8`) when the
//! client sends `Accept: text/toon`. Keeping this here guarantees the Worker and
//! the native service produce identical bytes for the same data.

use serde::Serialize;
use toon_format::{encode as toon_encode, EncodeOptions};

use crate::error::CoreError;

pub const CONTENT_TYPE_JSON: &str = "application/json";
pub const CONTENT_TYPE_TOON: &str = "text/toon; charset=utf-8";

/// Returns `true` when an `Accept` header value opts into TOON output.
pub fn wants_toon(accept: Option<&str>) -> bool {
    accept.map(|s| s.contains("text/toon")).unwrap_or(false)
}

/// Serialize `data` to JSON or TOON, returning the body and its content type.
pub fn encode<T: Serialize>(
    data: &T,
    use_toon: bool,
) -> Result<(String, &'static str), CoreError> {
    if use_toon {
        toon_encode(data, &EncodeOptions::new())
            .map(|body| (body, CONTENT_TYPE_TOON))
            .map_err(|e| CoreError::BadRequest(format!("TOON serialization error: {}", e)))
    } else {
        serde_json::to_string(data)
            .map(|body| (body, CONTENT_TYPE_JSON))
            .map_err(|e| CoreError::BadRequest(format!("JSON serialization error: {}", e)))
    }
}
