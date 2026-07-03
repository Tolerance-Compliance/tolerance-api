//! MCP tool shapes: definitions surfaced via `tools/list` and results
//! returned from `tools/call`.

use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

/// A tool advertised by `tools/list`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ToolDefinition {
    /// Stable tool name to pass to `tools/call`, e.g. `search_elements`.
    #[schema(value_type = String, example = "search_elements")]
    pub name: &'static str,
    /// What the tool does and when to reach for it.
    #[schema(value_type = String)]
    pub description: &'static str,
    /// JSON Schema for the tool's `arguments` object.
    #[serde(rename = "inputSchema")]
    #[schema(value_type = Object)]
    pub input_schema: Value,
}

/// One content block in a tool result. Always TOON-encoded text (the
/// consumer of a tool result is a model).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Content {
    /// Content type. Always `"text"`.
    #[serde(rename = "type")]
    #[schema(value_type = String, example = "text")]
    pub kind: &'static str,
    /// The tool output (TOON-encoded standards data).
    pub text: String,
}

impl Content {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            kind: "text",
            text: text.into(),
        }
    }
}

/// The result of `tools/call`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CallToolResult {
    /// The result content blocks (one TOON text block in practice).
    pub content: Vec<Content>,
    /// `true` when the tool reported a domain error (bad arguments, unknown
    /// document, …); the message is in `content`. Omitted when `false`.
    #[serde(rename = "isError", skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl CallToolResult {
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: false,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(message)],
            is_error: true,
        }
    }
}
