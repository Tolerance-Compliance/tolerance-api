//! `GET /v1/nist/documents`

use worker::{Request, Response, RouteContext};

use tolerance_api_core::service;

use crate::catalog;
use crate::error::ApiError;
use crate::runtime::{accept_toon, finish, respond};

pub async fn list(req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(list_impl(&req))
}

fn list_impl(req: &Request) -> Result<Response, ApiError> {
    let docs = service::list_nist_documents(&catalog::supported_nist_keys());
    respond(&docs, accept_toon(req))
}
