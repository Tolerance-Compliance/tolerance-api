//! Per-isolate singletons and small request/response helpers shared by handlers.

use std::sync::OnceLock;

use serde::Serialize;
use worker::{Request, Response, RouteContext};

use tolerance_api_core::encode::{encode, wants_toon};
use tolerance_api_core::model::DocumentKey;
use tolerance_api_core::poam::PoamValidator;
use tolerance_api_core::query::{parse_far_document_key, parse_nist_document_key, ElementQuery};
use tolerance_api_core::scoring::ScoringDatabase;

use crate::error::ApiError;

static SCORING: OnceLock<ScoringDatabase> = OnceLock::new();
static POAM: OnceLock<PoamValidator> = OnceLock::new();

/// Shared scoring database (built once per isolate from the embedded table).
pub fn scoring() -> &'static ScoringDatabase {
    SCORING.get_or_init(ScoringDatabase::new)
}

/// Shared POA&M validator (built once per isolate).
pub fn poam() -> &'static PoamValidator {
    POAM.get_or_init(|| PoamValidator::new(ScoringDatabase::new()))
}

/// Parse the `:document`/`:revision` params as a NIST document key.
pub fn nist_key(ctx: &RouteContext<()>) -> Result<DocumentKey, ApiError> {
    let document = ctx.param("document").map(String::as_str).unwrap_or_default();
    let revision = ctx.param("revision").map(String::as_str).unwrap_or_default();
    Ok(parse_nist_document_key(document, revision)?)
}

/// Parse the `:document`/`:revision` params as a FAR document key.
pub fn far_key(ctx: &RouteContext<()>) -> Result<DocumentKey, ApiError> {
    let document = ctx.param("document").map(String::as_str).unwrap_or_default();
    let revision = ctx.param("revision").map(String::as_str).unwrap_or_default();
    Ok(parse_far_document_key(document, revision)?)
}

/// A required path parameter (the route only matches when it is present).
pub fn param(ctx: &RouteContext<()>, name: &str) -> String {
    ctx.param(name).cloned().unwrap_or_default()
}

/// Whether the client requested TOON output via the `Accept` header.
pub fn accept_toon(req: &Request) -> bool {
    let accept = req.headers().get("Accept").ok().flatten();
    wants_toon(accept.as_deref())
}

/// Parse pagination/filter query parameters into an [`ElementQuery`].
pub fn element_query(req: &Request) -> ElementQuery {
    let mut query = ElementQuery::default();
    if let Ok(url) = req.url() {
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "type" => query.element_type = Some(value.into_owned()),
                "search" => query.search = Some(value.into_owned()),
                "limit" => query.limit = value.parse().ok(),
                "offset" => query.offset = value.parse().ok(),
                _ => {}
            }
        }
    }
    query
}

/// Encode a serializable value as JSON or TOON and build the HTTP response.
pub fn respond<T: Serialize>(data: &T, use_toon: bool) -> Result<Response, ApiError> {
    let (body, content_type) = encode(data, use_toon)?;
    let mut response = Response::ok(body)?;
    response.headers_mut().set("Content-Type", content_type)?;
    Ok(response)
}

/// Convert a handler `Result` into a `worker::Result<Response>`, rendering errors
/// as JSON responses with the right status code.
pub fn finish(result: Result<Response, ApiError>) -> worker::Result<Response> {
    match result {
        Ok(response) => Ok(response),
        Err(err) => err.into_response(),
    }
}
