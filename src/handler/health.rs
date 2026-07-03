use axum::{Json, http::StatusCode};
use serde_json::{Value, json};

use crate::constant;

/// Health check endpoint.
///
/// # Returns:
/// * `Json<Value>`: A JSON object containing the,
///                  health status,
///                  service name,
///                  version,
///                  current timestamp.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check successful", body = Value)
    ),
    tag = "Health"
)]
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status":    StatusCode::OK.as_u16(),
        "service":   constant::SERVICE_NAME,
        "version":   constant::VERSION,
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}
