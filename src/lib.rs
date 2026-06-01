//! Tolerance API - NIST SP 800-171 security requirements API.
//!
//! Pure modules (model, index, scoring, poam, assemble, response, kv::keys,
//! constant) compile for both the native `ingest` binary and the wasm Worker.
//! Worker-only modules (entry, routing, handlers, doc, kv::store) are gated to
//! the `wasm32` target so the native ingest build never pulls in `worker`.

pub mod cmmc;
pub mod constant;
pub mod kv;

#[cfg(target_arch = "wasm32")]
pub mod doc;
#[cfg(target_arch = "wasm32")]
pub mod handler;
#[cfg(target_arch = "wasm32")]
pub mod routing;

#[cfg(target_arch = "wasm32")]
mod entry;
