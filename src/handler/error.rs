use axum::{
    Json,
    http::StatusCode,
    response::{
        IntoResponse,
        Response
    },
};
use serde_json::Value;
use thiserror::Error;

/// API-specific errors for CMMC endpoints
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    /// The endpoint exists but does not support this document type.
    /// `hint` is a JSON object of suggested alternative endpoints.
    #[error("Not implemented: {message}")]
    NotImplemented { message: String, hint: serde_json::Value },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound(message) => {
                let body: Json<Value> = Json(serde_json::json!({
                    "error":   message,
                    "success": false,
                }));
                (StatusCode::NOT_FOUND, body).into_response()
            }
            ApiError::BadRequest(message) => {
                let body: Json<Value> = Json(serde_json::json!({
                    "error":   message,
                    "success": false,
                }));
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            ApiError::NotImplemented { message, hint } => {
                let body: Json<Value> = Json(serde_json::json!({
                    "error":   message,
                    "hint":    hint,
                    "success": false,
                }));
                (StatusCode::NOT_IMPLEMENTED, body).into_response()
            }
        }
    }
}
