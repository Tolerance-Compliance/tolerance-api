//! HTTP routing configuration

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::cmmc::handler::{
    get_element, get_element_relationships, get_elements, get_families, get_family,
    get_relationships, get_requirements, get_security_requirements, get_summary,
};
use crate::cmmc::CmmcState;
use crate::constant::{
    CMMC_ELEMENT_ENDPOINT, CMMC_ELEMENT_RELATIONS_ENDPOINT, CMMC_ELEMENTS_ENDPOINT,
    CMMC_FAMILIES_ENDPOINT, CMMC_FAMILY_ENDPOINT, CMMC_RELATIONSHIPS_ENDPOINT,
    CMMC_REQUIREMENTS_ENDPOINT, CMMC_SECURITY_REQS_ENDPOINT, CMMC_SUMMARY_ENDPOINT,
    HEALTH_ENDPOINT,
};
use crate::handler::health::health_check;

/// Creates a permissive CORS layer for development/testing purposes.
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false)
}

/// Build the application router with all routes
pub fn app(state: CmmcState) -> Router {
    Router::new()
        // Health check
        .route(HEALTH_ENDPOINT, get(health_check))
        // CMMC / NIST 800-171 endpoints
        .route(CMMC_SUMMARY_ENDPOINT, get(get_summary))
        .route(CMMC_FAMILIES_ENDPOINT, get(get_families))
        .route(CMMC_FAMILY_ENDPOINT, get(get_family))
        .route(CMMC_ELEMENTS_ENDPOINT, get(get_elements))
        .route(CMMC_ELEMENT_ENDPOINT, get(get_element))
        .route(CMMC_REQUIREMENTS_ENDPOINT, get(get_requirements))
        .route(CMMC_SECURITY_REQS_ENDPOINT, get(get_security_requirements))
        .route(CMMC_RELATIONSHIPS_ENDPOINT, get(get_relationships))
        .route(CMMC_ELEMENT_RELATIONS_ENDPOINT, get(get_element_relationships))
        // Middleware
        .with_state(state)
        .layer(create_cors_layer())
        .layer(TraceLayer::new_for_http())
}
