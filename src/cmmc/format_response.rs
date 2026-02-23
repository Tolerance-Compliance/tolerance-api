//! Format-aware response types that support both JSON and TOON output

use axum::{
    response::{IntoResponse, Response},
    http::{StatusCode, header},
};
use serde::Serialize;

use super::toon;

/// A response that can be serialized to either JSON or TOON format
pub struct FormatResponse<T> {
    data: T,
    use_toon: bool,
}

impl<T: Serialize> FormatResponse<T> {
    /// Create a new response that will use JSON format
    pub fn json(data: T) -> Self {
        Self {
            data,
            use_toon: false,
        }
    }

    /// Create a new response that will use TOON format
    pub fn toon(data: T) -> Self {
        Self {
            data,
            use_toon: true,
        }
    }

    /// Create a response with the specified format
    pub fn with_format(data: T, use_toon: bool) -> Self {
        Self { data, use_toon }
    }
}

impl<T: Serialize> IntoResponse for FormatResponse<T> {
    fn into_response(self) -> Response {
        if self.use_toon {
            // Serialize to TOON
            match toon::to_toon(&self.data) {
                Ok(toon_str) => {
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/toon; charset=utf-8")],
                        toon_str,
                    )
                        .into_response()
                }
                Err(e) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("TOON serialization error: {}", e),
                    )
                        .into_response()
                }
            }
        } else {
            // Serialize to JSON (default)
            match serde_json::to_string(&self.data) {
                Ok(json_str) => {
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "application/json")],
                        json_str,
                    )
                        .into_response()
                }
                Err(e) => {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("JSON serialization error: {}", e),
                    )
                        .into_response()
                }
            }
        }
    }
}
