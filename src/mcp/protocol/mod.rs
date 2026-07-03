//! Wire types: JSON-RPC 2.0 envelope and the MCP tool shapes.

pub mod jsonrpc;
pub mod tool;

pub use jsonrpc::{INVALID_PARAMS, INVALID_REQUEST, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use tool::{CallToolResult, Content, ToolDefinition};
