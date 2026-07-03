//! Handshake and capability-discovery responses: legacy `initialize`
//! (pre-2026-07-28) and modern `server/discover`.

use serde_json::{Value, json};

use super::constants::{
    LEGACY_FALLBACK_VERSION, LEGACY_PROTOCOL_VERSIONS, MODERN_PROTOCOL_VERSION,
    SERVER_INSTRUCTIONS, SERVER_NAME,
};

/// Legacy `initialize`: echo a mutually supported version.
pub fn initialize_result(params: &Value) -> Value {
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(LEGACY_FALLBACK_VERSION);
    let negotiated =
        if LEGACY_PROTOCOL_VERSIONS.contains(&requested) || requested == MODERN_PROTOCOL_VERSION {
            requested
        } else {
            LEGACY_FALLBACK_VERSION
        };

    result_body(negotiated)
}

/// Modern `server/discover` (2026-07-28): capability discovery without a
/// handshake or session.
pub fn discover_result() -> Value {
    result_body(MODERN_PROTOCOL_VERSION)
}

fn result_body(protocol_version: &str) -> Value {
    json!({
        "protocolVersion": protocol_version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
        "instructions": SERVER_INSTRUCTIONS,
    })
}
