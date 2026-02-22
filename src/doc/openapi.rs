//! OpenAPI specification and JSON endpoint.

use axum::{
    response::Response,
    http::{StatusCode, header}
};
use utoipa::OpenApi;
use serde_json;

use crate::constant::OPENAPI_CACHE_DURATION;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handler::health::health_check,
        crate::cmmc::handler::summary::get_summary,
        crate::cmmc::handler::families::get_families,
        crate::cmmc::handler::families::get_family,
        crate::cmmc::handler::elements::get_elements,
        crate::cmmc::handler::elements::get_element,
        crate::cmmc::handler::elements::get_requirements,
        crate::cmmc::handler::elements::get_security_requirements,
        crate::cmmc::handler::relationships::get_relationships,
        crate::cmmc::handler::relationships::get_element_relationships,
    ),
    components(
        schemas(
            crate::cmmc::response::DataSummary,
            crate::cmmc::response::Family,
            crate::cmmc::response::Requirement,
            crate::cmmc::response::SecurityRequirement,
            crate::cmmc::response::PaginatedResponse<crate::cmmc::model::Element>,
            crate::cmmc::model::Element,
            crate::cmmc::model::ElementType,
            crate::cmmc::model::Document,
            crate::cmmc::model::Relationship,
            crate::cmmc::CmmcLevel,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "CMMC", description = "CMMC/NIST SP 800-171 & 800-172 endpoints")
    ),
    info(
        title = "Tolerance API",
        version = env!("CARGO_PKG_VERSION"),
        description = "REST API for NIST SP 800-171 (CMMC L2) and SP 800-172 (CMMC L3) security requirements",
        license(name = "")
    )
)]
pub struct ApiDoc;

/// Serves the OpenAPI JSON specification.
///
/// Returns a JSON response containing the complete OpenAPI specification
/// for the Tolerance API, suitable for consumption by API documentation
/// tools and client generators.
pub async fn openapi_json() -> Response<axum::body::Body> {
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi)
        .expect("Failed to serialize OpenAPI specification");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, format!("public, max-age={}", OPENAPI_CACHE_DURATION))
        .body(axum::body::Body::from(json))
        .expect("Failed to build OpenAPI JSON response")
}
