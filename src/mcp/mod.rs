//! MCP integration for Claude Agent SDK.

pub mod manager;
pub mod rate_limiter;
pub mod schema;
pub mod server;
pub mod transport_factory;
pub mod transports;

pub use manager::{McpServer, McpServerManager, ToolInfo};
pub use rate_limiter::{RateLimitConfig, RateLimiter};
pub use schema::ToolDefinition;
pub use server::SdkMcpServer;
pub use transport_factory::create_mcp_server;
pub use transports::{HttpMcpServer, SseMcpServer, StdioMcpServer};
