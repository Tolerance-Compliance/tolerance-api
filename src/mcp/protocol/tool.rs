//! MCP tool shapes: definitions surfaced via `tools/list` and results
//! returned from `tools/call`.

use serde::Serialize;
use serde_json::Value;

/// A tool surfaced via `tools/list`.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// One piece of content in a tool result (text only — our results are TOON).
#[derive(Debug, Clone, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub kind: &'static str,
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
#[derive(Debug, Clone, Serialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
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
