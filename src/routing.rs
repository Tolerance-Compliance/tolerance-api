//! HTTP routing configuration

use axum::{routing::{get, post}, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::cmmc::handler::{
    get_documents,
    get_nist_element, get_nist_element_relationships, get_nist_elements,
    get_nist_families, get_nist_family, get_nist_relationships, get_nist_requirements,
    get_nist_security_requirements, get_nist_summary,
    get_far_element, get_far_element_relationships, get_far_elements,
    get_far_families, get_far_family, get_far_relationships, get_far_requirements,
    get_far_summary,
    validate_poam_requirement, validate_poam_batch, get_non_eligible_requirements,
};
use crate::cmmc::CmmcState;
use crate::constant::{
    HEALTH_ENDPOINT, MCP_ENDPOINT,
    NIST_DOCUMENTS_ENDPOINT, NIST_ELEMENT_ENDPOINT, NIST_ELEMENT_RELATIONS_ENDPOINT,
    NIST_ELEMENTS_ENDPOINT, NIST_FAMILIES_ENDPOINT, NIST_FAMILY_ENDPOINT,
    NIST_RELATIONSHIPS_ENDPOINT, NIST_REQUIREMENTS_ENDPOINT, NIST_SECURITY_REQS_ENDPOINT,
    NIST_SUMMARY_ENDPOINT,
    POAM_VALIDATE_REQ_ENDPOINT, POAM_VALIDATE_BATCH_ENDPOINT, POAM_NON_ELIGIBLE_REQS_ENDPOINT,
    FAR_ELEMENT_ENDPOINT, FAR_ELEMENT_RELATIONS_ENDPOINT,
    FAR_ELEMENTS_ENDPOINT, FAR_FAMILIES_ENDPOINT, FAR_FAMILY_ENDPOINT,
    FAR_RELATIONSHIPS_ENDPOINT, FAR_REQUIREMENTS_ENDPOINT, FAR_SUMMARY_ENDPOINT,
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
        // MCP (Model Context Protocol) tool surface — see src/mcp/mod.rs
        .route(MCP_ENDPOINT,                    post(crate::mcp::handle_mcp))
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
        // POA&M validation API
        .route(POAM_VALIDATE_REQ_ENDPOINT,      get(validate_poam_requirement))
        .route(POAM_VALIDATE_BATCH_ENDPOINT,    post(validate_poam_batch))
        .route(POAM_NON_ELIGIBLE_REQS_ENDPOINT, get(get_non_eligible_requirements))
        // FAR API
        .route(FAR_SUMMARY_ENDPOINT,            get(get_far_summary))
        .route(FAR_FAMILIES_ENDPOINT,           get(get_far_families))
        .route(FAR_FAMILY_ENDPOINT,             get(get_far_family))
        .route(FAR_ELEMENTS_ENDPOINT,           get(get_far_elements))
        .route(FAR_ELEMENT_ENDPOINT,            get(get_far_element))
        .route(FAR_REQUIREMENTS_ENDPOINT,       get(get_far_requirements))
        .route(FAR_RELATIONSHIPS_ENDPOINT,      get(get_far_relationships))
        .route(FAR_ELEMENT_RELATIONS_ENDPOINT,  get(get_far_element_relationships))
        // Middleware
        .with_state(state)
        .layer(create_cors_layer())
        .layer(TraceLayer::new_for_http())
}
