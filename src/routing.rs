//! HTTP routing configuration

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::cmmc::handler::{
    get_documents, get_nist_element, get_nist_element_relationships, get_nist_elements,
    get_nist_families, get_nist_family, get_nist_relationships, get_nist_requirements,
    get_nist_security_requirements, get_nist_summary,
};
use crate::cmmc::CmmcState;
use crate::constant::{
    HEALTH_ENDPOINT,
    NIST_DOCUMENTS_ENDPOINT, NIST_ELEMENT_ENDPOINT, NIST_ELEMENT_RELATIONS_ENDPOINT,
    NIST_ELEMENTS_ENDPOINT, NIST_FAMILIES_ENDPOINT, NIST_FAMILY_ENDPOINT,
    NIST_RELATIONSHIPS_ENDPOINT, NIST_REQUIREMENTS_ENDPOINT, NIST_SECURITY_REQS_ENDPOINT,
    NIST_SUMMARY_ENDPOINT,
};
use crate::doc::{favicon, openapi_json, swagger_ui};
use crate::handler::health::health_check;

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
        // Documentation
        .route("/",                        get(swagger_ui))
        .route("/api-docs/openapi.json",   get(openapi_json))
        .route("/favicon.ico",             get(favicon))
        // Health check
        .route(HEALTH_ENDPOINT,                 get(health_check))
        // NIST API
        .route(NIST_DOCUMENTS_ENDPOINT,         get(get_documents))
        .route(NIST_SUMMARY_ENDPOINT,           get(get_nist_summary))
        .route(NIST_FAMILIES_ENDPOINT,          get(get_nist_families))
        .route(NIST_FAMILY_ENDPOINT,            get(get_nist_family))
        .route(NIST_ELEMENTS_ENDPOINT,          get(get_nist_elements))
        .route(NIST_ELEMENT_ENDPOINT,           get(get_nist_element))
        .route(NIST_REQUIREMENTS_ENDPOINT,      get(get_nist_requirements))
        .route(NIST_SECURITY_REQS_ENDPOINT,     get(get_nist_security_requirements))
        .route(NIST_RELATIONSHIPS_ENDPOINT,     get(get_nist_relationships))
        .route(NIST_ELEMENT_RELATIONS_ENDPOINT, get(get_nist_element_relationships))
        // Middleware
        .with_state(state)
        .layer(create_cors_layer())
        .layer(TraceLayer::new_for_http())
}
