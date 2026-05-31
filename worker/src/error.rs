//! Worker request error type.
//!
//! Mirrors the native `ApiError` JSON shapes/status codes so responses are
//! identical, plus an `Internal` variant for Worker-runtime/R2 failures.

use serde_json::{json, Value};
use worker::Response;

use tolerance_api_core::error::CoreError;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    NotImplemented { message: String, hint: Value },
    Internal(String),
}

impl ApiError {
    fn status(&self) -> u16 {
        match self {
            ApiError::NotFound(_) => 404,
            ApiError::BadRequest(_) => 400,
            ApiError::NotImplemented { .. } => 501,
            ApiError::Internal(_) => 500,
        }
    }

    fn body(&self) -> Value {
        match self {
            ApiError::NotFound(message)
            | ApiError::BadRequest(message)
            | ApiError::Internal(message) => json!({ "error": message, "success": false }),
            ApiError::NotImplemented { message, hint } => {
                json!({ "error": message, "hint": hint, "success": false })
            }
        }
    }

    /// Render the error as a JSON response with the matching status code.
    pub fn into_response(self) -> worker::Result<Response> {
        Ok(Response::from_json(&self.body())?.with_status(self.status()))
    }
}

/// Map the framework-agnostic core error one-to-one.
impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NotFound(message) => ApiError::NotFound(message),
            CoreError::BadRequest(message) => ApiError::BadRequest(message),
            CoreError::NotImplemented { message, hint } => {
                ApiError::NotImplemented { message, hint }
            }
        }
    }
}

/// Treat any underlying Worker runtime error as an internal failure.
impl From<worker::Error> for ApiError {
    fn from(err: worker::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}
