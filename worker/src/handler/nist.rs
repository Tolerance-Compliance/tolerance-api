//! `/v1/nist/:document/:revision/*`

use worker::{Request, Response, RouteContext};

use tolerance_api_core::query::require_cmmc_structured;
use tolerance_api_core::service;

use crate::catalog;
use crate::error::ApiError;
use crate::runtime::{accept_toon, element_query, finish, nist_key, param, poam, respond, scoring};

pub async fn summary(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(summary_impl(req, ctx).await)
}

async fn summary_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    let data = catalog::load(&ctx.env, key).await?;
    respond(&service::summary(&data.context()), accept_toon(&req))
}

pub async fn families(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(families_impl(req, ctx).await)
}

async fn families_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    require_cmmc_structured(key)?;
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::families(&data.context(), scoring(), poam());
    respond(&out, accept_toon(&req))
}

pub async fn family(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(family_impl(req, ctx).await)
}

async fn family_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    require_cmmc_structured(key)?;
    let id = param(&ctx, "id");
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::family(&data.context(), scoring(), poam(), &id)?;
    respond(&out, accept_toon(&req))
}

pub async fn elements(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(elements_impl(req, ctx).await)
}

async fn elements_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    let query = element_query(&req);
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::elements(&data.context(), &query);
    respond(&out, accept_toon(&req))
}

pub async fn element(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(element_impl(req, ctx).await)
}

async fn element_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    let id = param(&ctx, "id");
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::element(&data.context(), &id)?;
    respond(&out, accept_toon(&req))
}

pub async fn requirements(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(requirements_impl(req, ctx).await)
}

async fn requirements_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    require_cmmc_structured(key)?;
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::requirements(&data.context(), scoring(), poam());
    respond(&out, accept_toon(&req))
}

pub async fn security_requirements(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(security_requirements_impl(req, ctx).await)
}

async fn security_requirements_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    require_cmmc_structured(key)?;
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::security_requirements(&data.context(), scoring(), poam());
    respond(&out, accept_toon(&req))
}

pub async fn relationships(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(relationships_impl(req, ctx).await)
}

async fn relationships_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    let data = catalog::load(&ctx.env, key).await?;
    respond(&service::relationships(&data.context()), accept_toon(&req))
}

pub async fn element_relationships(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(element_relationships_impl(req, ctx).await)
}

async fn element_relationships_impl(req: Request, ctx: RouteContext<()>) -> Result<Response, ApiError> {
    let key = nist_key(&ctx)?;
    let id = param(&ctx, "id");
    let data = catalog::load(&ctx.env, key).await?;
    let out = service::element_relationships(&data.context(), &id)?;
    respond(&out, accept_toon(&req))
}
