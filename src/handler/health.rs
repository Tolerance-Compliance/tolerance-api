use axum::{
    Json,
    http::StatusCode,
};
use serde_json::{json, Value};

use crate::constant;

/// Health check endpoint.
///
/// # Returns:
/// * `Json<Value>`: A JSON object containing the,
///                  health status,
///                  service name,
///                  version,
///                  current timestamp.
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status":    StatusCode::OK.as_u16(),
        "service":   constant::SERVICE_NAME,
        "version":   constant::VERSION,
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}