use axum::{
    Router,
    routing::get,
};
use tower_http::cors::{
    CorsLayer,
    Any
};
use tower_http::trace::TraceLayer;

use crate::constant::{
    HEALTH_ENDPOINT,
    CMMC_SUMMARY_ENDPOINT,
    CMMC_FAMILIES_ENDPOINT,
    CMMC_FAMILY_ENDPOINT,
    CMMC_ELEMENTS_ENDPOINT,
    CMMC_ELEMENT_ENDPOINT,
    CMMC_REQUIREMENTS_ENDPOINT,
    CMMC_SECURITY_REQS_ENDPOINT,
    CMMC_RELATIONSHIPS_ENDPOINT,
    CMMC_ELEMENT_RELATIONS_ENDPOINT,
};
use crate::handler::health::health_check;
use crate::handler::cmmc::{
    CmmcState,
    get_summary,
    get_families,
    get_family,
    get_elements,
    get_element,
    get_requirements,
    get_security_requirements,
    get_relationships,
    get_element_relationships,
};

/// Creates a permissive CORS layer for development/testing purposes.
///
/// This configuration allows:
/// * Any origin to make requests.
/// * Any HTTP methods (GET, POST, etc.).
/// * Any headers in requests.
/// * Disables credentials whether that be cookies or authorization.
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false)
}

pub fn app(state: CmmcState) -> Router {
    Router::new()
        .route(HEALTH_ENDPOINT,               get(health_check))
        // CMMC / NIST 800-171 endpoints
        .route(CMMC_SUMMARY_ENDPOINT,         get(get_summary))
        .route(CMMC_FAMILIES_ENDPOINT,        get(get_families))
        .route(CMMC_FAMILY_ENDPOINT,          get(get_family))
        .route(CMMC_ELEMENTS_ENDPOINT,        get(get_elements))
        .route(CMMC_ELEMENT_ENDPOINT,         get(get_element))
        .route(CMMC_REQUIREMENTS_ENDPOINT,    get(get_requirements))
        .route(CMMC_SECURITY_REQS_ENDPOINT,   get(get_security_requirements))
        .route(CMMC_RELATIONSHIPS_ENDPOINT,   get(get_relationships))
        .route(CMMC_ELEMENT_RELATIONS_ENDPOINT, get(get_element_relationships))

        .with_state(state)                   // Injects CMMC data state
        .layer(create_cors_layer())          // Enabled CORS, allows web browsers to use this.
        .layer(TraceLayer::new_for_http())   // Automatic HTTP request/response logging.
}
