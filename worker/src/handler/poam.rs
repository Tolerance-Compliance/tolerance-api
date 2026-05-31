//! POA&M validation endpoints under `/v1/nist/:document/:revision/poam/*`

use worker::{Request, Response, RouteContext};

use tolerance_api_core::service::{self, BatchValidationRequest};

use crate::catalog;
use crate::error::ApiError;
use crate::runtime::{accept_toon, finish, nist_key, param, poam, respond};

pub async fn validate_requirement(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(validate_requirement_impl(req, ctx).await)
}

async fn validate_requirement_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    let requirement_id = param(&ctx, "requirement_id");
    // Verify the document is loaded.
    catalog::load(&ctx.env, key).await?;
    let out = service::validate_poam_requirement(poam(), &requirement_id);
    respond(&out, accept_toon(&req))
}

pub async fn validate_batch(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let body = req.json::<BatchValidationRequest>().await;
    finish(validate_batch_impl(req, ctx, body).await)
}

async fn validate_batch_impl(
    req: Request,
    ctx: RouteContext<()>,
    body: worker::Result<BatchValidationRequest>,
) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    // Verify the document is loaded.
    catalog::load(&ctx.env, key).await?;
    let request =
        body.map_err(|e| ApiError::BadRequest(format!("invalid request body: {}", e)))?;
    let out = service::validate_poam_batch(poam(), &request.requirement_ids);
    respond(&out, accept_toon(&req))
}

pub async fn non_eligible(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(non_eligible_impl(req, ctx).await)
}

async fn non_eligible_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    // Verify the document is loaded.
    catalog::load(&ctx.env, key).await?;
    let out = service::non_eligible(poam());
    respond(&out, accept_toon(&req))
}
