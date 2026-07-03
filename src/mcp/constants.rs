//! Every constant the MCP surface uses, in one place.

/// The stateless protocol revision this server is built for.
pub const MODERN_PROTOCOL_VERSION: &str = "2026-07-28";

/// Handshake-era protocol revisions we answer `initialize` for.
pub const LEGACY_PROTOCOL_VERSIONS: &[&str] =
    &["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"];

/// Fallback version offered when a legacy client requests something unknown;
/// per spec the client decides whether it can proceed.
pub const LEGACY_FALLBACK_VERSION: &str = "2025-11-25";

/// Server name reported in `serverInfo`.
pub const SERVER_NAME: &str = "tolerance-api";

/// Usage hint returned from `initialize` and `server/discover`.
pub const SERVER_INSTRUCTIONS: &str = "Read-only tools over the official NIST SP 800-53/171/171A/172 \
     and FAR 52.204-21 corpora plus CMMC scoring and POA&M eligibility rules. Start with \
     list_documents to discover valid document/revision arguments. Results are TOON-encoded text.";

/// SEP-2243 routable headers (lowercase; `HeaderMap` lookups are
/// case-insensitive but store lowercase).
pub const HEADER_MCP_METHOD: &str = "mcp-method";
pub const HEADER_MCP_NAME: &str = "mcp-name";
pub const HEADER_MCP_PROTOCOL_VERSION: &str = "mcp-protocol-version";

/// Default / maximum result counts for `search_elements`.
pub const SEARCH_DEFAULT_LIMIT: usize = 20;
pub const SEARCH_MAX_LIMIT: usize = 200;

/// Cap on relationship-linked elements resolved by `get_element`.
pub const RELATED_ELEMENTS_LIMIT: usize = 50;

/// Shared JSON-schema parameter descriptions for `document` / `revision`.
pub const DOC_PARAM_DESC: &str = "Document: `sp800-53`, `sp800-53a`, `sp800-53b`, `sp800-171`, \
     `sp800-171a`, `sp800-172`, `sp800-172a`, or `52.204-21` (FAR)";
pub const REV_PARAM_DESC: &str =
    "Revision: `r5` (800-53*), `r2`/`r3` (171/171a), `v1` (172/172a), `v2` (FAR)";
