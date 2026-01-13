//! MCP integration for Claude Agent SDK.

pub mod manager;
pub mod schema;
pub mod transports;

pub use manager::{McpServer, McpServerManager, ToolInfo};
pub use schema::ToolDefinition;
pub use transports::StdioMcpServer;
pub mod server;
pub use server::SdkMcpServer;
