//! CORS helpers applied to every Worker response.

use worker::{Response, Result};

/// Attach permissive CORS headers to a response (matches the native
/// `CorsLayer::allow_origin(Any)` configuration).
pub fn wrap(mut response: Response) -> Result<Response> {
    let headers = response.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, Accept")?;
    headers.set("Access-Control-Max-Age", "86400")?;
    Ok(response)
}

/// Respond to a CORS preflight (`OPTIONS`) request.
pub fn preflight() -> Result<Response> {
    wrap(Response::empty()?.with_status(204))
}
