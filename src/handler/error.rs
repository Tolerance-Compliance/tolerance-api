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

#[derive(Error, Debug)]
pub enum ErrorHandler {
    #[error("Conflict error: {0}")]
    Conflict(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("I'm a teapot: {0}")]
    ImATeapot(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Bundle not found: {0}")]
    BundleNotFound(String),
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
    #[error("Incompatible version: current={current}, required minimum={required}")]
    IncompatibleVersion { current: String, required: String },
    #[error("Incompatible hardware: device={device}, required={required}")]
    IncompatibleHardware { device: String, required: String },
    #[error("Invalid channel: {0}")]
    InvalidChannel(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// API-specific errors for CMMC endpoints
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ErrorHandler {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ErrorHandler::Conflict(ref message) => {
                (StatusCode::CONFLICT, message.clone())
            }
            ErrorHandler::InvalidRequest(ref message) => {
                (StatusCode::BAD_REQUEST, message.clone())
            },
            ErrorHandler::ImATeapot(ref message) => {
                (StatusCode::IM_A_TEAPOT, message.clone())
            },
            ErrorHandler::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string())
            },
            ErrorHandler::BundleNotFound(ref message) => {
                (StatusCode::NOT_FOUND, message.clone())
            },
            ErrorHandler::InvalidVersion(ref message) => {
                (StatusCode::BAD_REQUEST, message.clone())
            },
            ErrorHandler::IncompatibleVersion { ref current, ref required } => {
                (StatusCode::BAD_REQUEST,
                 format!("Current version '{}' incompatible. Minimum required: {}", current, required))
            },
            ErrorHandler::IncompatibleHardware { ref device, ref required } => {
                (StatusCode::BAD_REQUEST,
                 format!("Device hardware '{}' incompatible. Required: {}", device, required))
            },
            ErrorHandler::InvalidChannel(ref message) => {
                (StatusCode::BAD_REQUEST, message.clone())
            },
            ErrorHandler::Configuration(ref message) => {
                tracing::error!("Configuration error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Server configuration error".to_string())
            },
            ErrorHandler::Internal(ref message) => {
                tracing::error!("Internal error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
        };

        let body: Json<Value> = Json(serde_json::json!({
            "error":   error_message,
            "success": false
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(ref message) => {
                (StatusCode::NOT_FOUND, message.clone())
            }
            ApiError::BadRequest(ref message) => {
                (StatusCode::BAD_REQUEST, message.clone())
            }
        };

        let body: Json<Value> = Json(serde_json::json!({
            "error":   error_message,
            "success": false
        }));

        (status, body).into_response()
    }
}