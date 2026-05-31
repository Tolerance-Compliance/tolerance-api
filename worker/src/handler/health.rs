//! `GET /health`

use worker::{Date, Request, Response, RouteContext};

use crate::error::ApiError;
use crate::runtime::finish;
use crate::{SERVICE_NAME, VERSION};

pub async fn health(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    finish(health_impl())
}

fn health_impl() -> Result<Response, ApiError> {
    let body = serde_json::json!({
        "status": 200,
        "service": SERVICE_NAME,
        "version": VERSION,
        "timestamp": Date::now().as_millis(),
    });
    Ok(Response::from_json(&body)?)
}
