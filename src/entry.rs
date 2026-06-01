//! Cloudflare Worker entry point (wasm target).
//!
//! Replaces the native `main.rs` server. Builds the axum `Router`, injects the
//! `DOCS` KV namespace as state, and drives the router as a `tower` service.

use tower_service::Service;
use worker::{event, Context, Env, HttpRequest, Result};

use crate::kv::store::AppState;
use crate::routing::app;

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();

    let kv = env.kv("DOCS")?;
    let state = AppState::new(kv);

    Ok(app(state).call(req).await?)
}
