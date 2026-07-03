//! Tests for the MCP surface, mirroring the source layout:
//!
//! - [`fixtures`] — shared state builder and request helpers
//! - [`tools`] — tool dispatch over real catalog data
//! - [`handler`] — protocol behavior through the real router (legacy
//!   handshake, modern statelessness, SEP-2243 header validation)

mod fixtures;
mod handler;
mod tools;
