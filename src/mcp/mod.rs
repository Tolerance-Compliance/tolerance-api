//! MCP (Model Context Protocol) surface over the loaded catalog (TOL-767).
//!
//! One in-process `POST /mcp` endpoint exposes the same in-memory
//! [`CmmcState`](crate::cmmc::CmmcState) the REST handlers read — no separate
//! service, no HTTP self-calls. Everything here is **read-only public
//! standards text** (NIST SP 800-53/171/171A/172, FAR 52.204-21, CMMC
//! scoring/POA&M rules): no CUI, no tenant data, so the endpoint carries the
//! same exposure as the existing REST API.
//!
//! ## Protocol posture (researched July 2026)
//!
//! The server is stateless by construction (immutable in-memory data), which
//! matches the direction of the spec: the 2026-07-28 revision removes the
//! `initialize` handshake and protocol sessions entirely (SEP-2575/2567) and
//! requires routable `Mcp-Method`/`Mcp-Name` headers (SEP-2243). We speak
//! both eras:
//!
//! - **Modern**: every request is self-contained; `server/discover` answers
//!   capability discovery; when the SEP-2243 headers or the
//!   `MCP-Protocol-Version` header are present they are validated against the
//!   body and a mismatch is rejected (400, JSON-RPC error).
//! - **Legacy**: `initialize` is answered (echoing a supported version) and
//!   nothing session-like is required afterwards, so 2024-11-05 …
//!   2025-11-25 clients (today's Cursor/Claude) work unchanged.
//!
//! JSON-object responses only — every tool answers immediately from memory,
//! so the SSE side of Streamable HTTP is unnecessary. `Origin` is
//! deliberately unrestricted, matching the API's existing public, read-only,
//! CORS-`Any` posture.
//!
//! Tool results are returned as TOON text (the repo's LLM-optimized
//! encoding, 30-40% fewer tokens than JSON) — the consumer of a tool result
//! is always a model.

//! ## Module layout
//!
//! - [`constants`] — every protocol version, header name, limit, and shared
//!   description string
//! - [`protocol`] — wire types (JSON-RPC envelope, tool shapes)
//! - [`handler`] — the axum entry point: parse → validate → dispatch
//! - [`headers`] — SEP-2243 header/body validation
//! - [`discovery`] — `initialize` / `server/discover` responses
//! - [`tools`] — the tool registry; **one file per tool**, shared helpers in
//!   `tools::support`. Adding a tool = one new file + one registry line.

pub mod constants;
pub mod discovery;
pub mod handler;
pub mod headers;
pub mod protocol;
pub mod tools;

#[cfg(test)]
mod tests;

pub use handler::handle_mcp;
