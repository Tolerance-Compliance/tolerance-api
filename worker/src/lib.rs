//! Cloudflare Worker edge for the CMMC / NIST API.
//!
//! Routes mirror the native axum service (same endpoint path templates from
//! `tolerance_api_core::endpoints`) and dispatch to handlers that reuse the
//! framework-agnostic `tolerance_api_core::service` logic. Catalog data is loaded
//! lazily from the `CATALOGS` R2 bucket and cached per isolate. SP 800-53 is not
//! served here (it stays on the native Fly service), and Swagger/OpenAPI is
//! native-only.

use worker::*;

use tolerance_api_core::endpoints::{
    FAR_ELEMENTS_ENDPOINT, FAR_ELEMENT_ENDPOINT, FAR_ELEMENT_RELATIONS_ENDPOINT,
    FAR_FAMILIES_ENDPOINT, FAR_FAMILY_ENDPOINT, FAR_RELATIONSHIPS_ENDPOINT,
    FAR_REQUIREMENTS_ENDPOINT, FAR_SUMMARY_ENDPOINT, HEALTH_ENDPOINT, NIST_DOCUMENTS_ENDPOINT,
    NIST_ELEMENTS_ENDPOINT, NIST_ELEMENT_ENDPOINT, NIST_ELEMENT_RELATIONS_ENDPOINT,
    NIST_FAMILIES_ENDPOINT, NIST_FAMILY_ENDPOINT, NIST_RELATIONSHIPS_ENDPOINT,
    NIST_REQUIREMENTS_ENDPOINT, NIST_SECURITY_REQS_ENDPOINT, NIST_SUMMARY_ENDPOINT,
    POAM_NON_ELIGIBLE_REQS_ENDPOINT, POAM_VALIDATE_BATCH_ENDPOINT, POAM_VALIDATE_REQ_ENDPOINT,
};

mod catalog;
mod cors;
mod error;
mod handler;
mod runtime;

/// Reported in `/health`. Kept equal to the native service name for parity.
pub const SERVICE_NAME: &str = "tolerance-api";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    if req.method() == Method::Options {
        return cors::preflight();
    }

    let router = Router::new()
        .get_async(HEALTH_ENDPOINT, handler::health::health)
        .get_async(NIST_DOCUMENTS_ENDPOINT, handler::documents::list)
        // NIST
        .get_async(NIST_SUMMARY_ENDPOINT, handler::nist::summary)
        .get_async(NIST_FAMILIES_ENDPOINT, handler::nist::families)
        .get_async(NIST_FAMILY_ENDPOINT, handler::nist::family)
        .get_async(NIST_ELEMENTS_ENDPOINT, handler::nist::elements)
        .get_async(NIST_ELEMENT_ENDPOINT, handler::nist::element)
        .get_async(NIST_REQUIREMENTS_ENDPOINT, handler::nist::requirements)
        .get_async(NIST_SECURITY_REQS_ENDPOINT, handler::nist::security_requirements)
        .get_async(NIST_RELATIONSHIPS_ENDPOINT, handler::nist::relationships)
        .get_async(NIST_ELEMENT_RELATIONS_ENDPOINT, handler::nist::element_relationships)
        // POA&M
        .get_async(POAM_VALIDATE_REQ_ENDPOINT, handler::poam::validate_requirement)
        .post_async(POAM_VALIDATE_BATCH_ENDPOINT, handler::poam::validate_batch)
        .get_async(POAM_NON_ELIGIBLE_REQS_ENDPOINT, handler::poam::non_eligible)
        // FAR
        .get_async(FAR_SUMMARY_ENDPOINT, handler::far::summary)
        .get_async(FAR_FAMILIES_ENDPOINT, handler::far::families)
        .get_async(FAR_FAMILY_ENDPOINT, handler::far::family)
        .get_async(FAR_ELEMENTS_ENDPOINT, handler::far::elements)
        .get_async(FAR_ELEMENT_ENDPOINT, handler::far::element)
        .get_async(FAR_REQUIREMENTS_ENDPOINT, handler::far::requirements)
        .get_async(FAR_RELATIONSHIPS_ENDPOINT, handler::far::relationships)
        .get_async(FAR_ELEMENT_RELATIONS_ENDPOINT, handler::far::element_relationships);

    let response = match router.run(req, env).await {
        Ok(response) => response,
        Err(err) => error::ApiError::from(err).into_response()?,
    };

    cors::wrap(response)
}
