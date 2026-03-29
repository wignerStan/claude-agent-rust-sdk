//! # Claude Agent MCP SDK
//!
//! In-process MCP (Model Context Protocol) server support for the Claude Agent SDK.
//!
//! This crate allows you to define custom tools that Claude can call, running as
//! an MCP server subprocess that communicates via JSON-RPC over stdio.
//!
//! ## Quick Start
//!
//! ```ignore
//! use claude_agent_mcp_sdk::server::SdkMcpServer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut server = SdkMcpServer::new("my-tools", "0.1.0");
//!
//!     server.tool(
//!         "greet",
//!         "Greets the user by name",
//!         serde_json::json!({
//!             "type": "object",
//!             "properties": {
//!                 "name": { "type": "string", "description": "The name to greet" }
//!             },
//!             "required": ["name"]
//!         }),
//!         |input| Box::pin(async move {
//!             let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("World");
//!             Ok(serde_json::json!({
//!                 "content": [{ "type": "text", "text": format!("Hello, {}!", name) }]
//!             }))
//!         }),
//!     );
//!
//!     server.run_stdio().await.unwrap();
//! }
//! ```

pub mod error;
pub mod server;
pub mod tool;

/// Re-exports of commonly used types.
pub mod reexports {
    pub use serde_json;
}

pub use error::McpSdkError;
pub use server::{McpSdkServerConfig, SdkMcpServer};
pub use tool::{tool, ToolDefinition, ToolHandler};
