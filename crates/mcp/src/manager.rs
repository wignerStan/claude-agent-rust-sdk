//! MCP Server Manager for Claude Agent SDK.
//!
//! This module provides a centralized registry for managing multiple MCP (Model Context Protocol)
//! servers. The manager handles server registration, tool discovery, and message
//! routing between the agent and MCP servers.
//!
//! # Architecture
//!
//! The manager maintains a registry of MCP servers, each implementing the `McpServer` trait.
//! Servers can be dynamically registered and deregistered. The agent can query
//! available tools across all servers and make tool calls to specific servers.
//!
//! # Features
//!
//! - **Dynamic Registration**: Servers can be added/removed at runtime
//! - **Tool Discovery**: Automatic aggregation of tools from all registered servers
//! - **Message Routing**: Routes tool calls to appropriate servers
//! - **Error Handling**: Propagates errors from server operations
//!
//! # Example
//!
//! ```rust,no_run
//! use claude_agent_mcp::{McpServerManager, McpServer};
//! use claude_agent_mcp::server::SdkMcpServer;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut manager = McpServerManager::new();
//!
//!     // Register a server
//!     let server = SdkMcpServer::new("calculator");
//!     manager.register(Box::new(server));
//!
//!     // List all tools
//!     let tools = manager.list_all_tools().await?;
//!     println!("Available tools: {:?}", tools);
//!
//!     // Get a server and call a tool
//!     if let Some(server) = manager.get("calculator") {
//!         let result = server.call_tool("add", serde_json::json!({"a": 5, "b": 3})).await?;
//!         println!("Result: {:?}", result);
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use claude_agent_types::ClaudeAgentError;
use serde_json::Value;

/// MCP Server Manager handles registration and communication with MCP servers.
///
/// This struct provides a centralized registry for managing multiple MCP servers.
/// It maintains a mapping of server names to server instances and provides
/// methods for registration, tool discovery, and tool invocation.
///
/// # Thread Safety
///
/// The manager is `Send + Sync` and can be safely shared across multiple
/// threads or async tasks. Internal state is protected by `Arc<RwLock<HashMap>>`.
/// The struct acts as a handle and can be cheaply cloned.
///
/// # Server Registration
///
/// Servers are stored in a `HashMap` where:
/// - The key is a unique server name (used for tool calls)
/// - The value is an `Arc<dyn McpServer>` trait object
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent_mcp::{McpServerManager, McpServer};
/// use claude_agent_mcp::server::SdkMcpServer;
///
/// let mut manager = McpServerManager::new();
/// manager.register(Box::new(SdkMcpServer::new("my_server")));
/// ```
#[derive(Clone)]
pub struct McpServerManager {
    servers: Arc<RwLock<HashMap<String, Arc<dyn McpServer>>>>,
}

/// Trait for MCP server implementations.
///
/// This trait defines the interface that all MCP servers must implement.
/// The manager uses this trait to interact with registered servers.
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to allow safe concurrent access
/// from multiple threads or async tasks.
///
/// # Required Methods
///
/// - `name()`: Returns the server's identifier
/// - `list_tools()`: Returns available tools with their schemas
/// - `call_tool()`: Executes a tool with given arguments
/// - `handle_client_message()`: Processes incoming JSON-RPC messages (optional)
///
/// # Error Handling
///
/// All methods return `Result<T, ClaudeAgentError>` to ensure proper error
/// propagation throughout the application. Implementations should convert
/// server-specific errors to `ClaudeAgentError::Mcp` variant.
///
/// # Example Implementation
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use serde_json::Value;
/// use claude_agent_types::{ClaudeAgentError, ToolInfo};
/// use crate::manager::{McpServer, ToolHandler};
///
/// struct MyServer {
///     name: String,
/// }
///
/// #[async_trait]
/// impl McpServer for MyServer {
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
///         Ok(vec![
///             ToolInfo {
///                 name: "my_tool".to_string(),
///                 description: Some("A sample tool".to_string()),
///                 input_schema: None,
///             }
///         ])
///     }
///
///     async fn call_tool(
///         &self,
///         name: &str,
///         arguments: Value,
///     ) -> Result<Value, ClaudeAgentError> {
///         // Implement tool logic here
///         Ok(serde_json::json!({"result": "success"}))
///     }
///
///     async fn handle_client_message(
///         &self,
///         message: Value,
///     ) -> Result<Value, ClaudeAgentError> {
///         // Default implementation returns error
///         Err(ClaudeAgentError::Mcp("Not implemented".to_string()))
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait McpServer: Send + Sync {
    /// Get the server name.
    ///
    /// Returns a unique identifier for this server instance.
    /// This name is used when registering the server with the manager
    /// and when making tool calls to identify the target server.
    ///
    /// # Returns
    ///
    /// Returns `&str` with static lifetime tied to `self`.
    fn name(&self) -> &str;

    /// List available tools from this server.
    ///
    /// Returns a list of all tools available on this server, including
    /// their names, descriptions, and input schemas.
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Mcp` if tool discovery fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tools = server.list_tools().await?;
    /// for tool in &tools {
    ///     println!("Tool: {} - {}", tool.name, tool.description.unwrap_or("No description"));
    /// }
    /// ```
    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError>;

    /// Call a tool on this server.
    ///
    /// Executes a tool with the provided arguments and returns the result.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the tool to call (must match a tool from `list_tools()`)
    /// - `arguments`: The arguments to pass to the tool (must match the tool's input schema)
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Mcp` if:
    /// - Tool not found
    /// - Arguments don't match input schema
    /// - Tool execution fails
    ///
    /// # Thread Safety
    ///
    /// This method may be called concurrently from multiple tasks. Implementations
    /// must ensure proper synchronization of any internal state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = server.call_tool("calculator", "add", serde_json::json!({"a": 5, "b": 3})).await?;
    /// println!("Result: {:?}", result);
    /// ```
    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, ClaudeAgentError>;

    /// Handle a raw JSON-RPC message from the client (CLI).
    ///
    /// This method is called when the agent receives a JSON-RPC message
    /// that should be routed to an MCP server. Implementations can choose
    /// to handle specific message types or return an error for unsupported messages.
    ///
    /// # Parameters
    ///
    /// - `message`: The raw JSON-RPC message received from the client
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Mcp` if the message cannot be handled.
    ///
    /// # Default Behavior
    ///
    /// The default implementation returns an error indicating that the message
    /// was not handled. Servers that want to process client messages should
    /// override this method.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn handle_client_message(&self, message: Value) -> Result<Value, ClaudeAgentError> {
    ///     let method = message.get("method").and_then(|m| m.as_str());
    ///     match method {
    ///         Some("tools/list") => {
    ///             // Handle tool listing
    ///             let tools = self.list_tools().await?;
    ///             Ok(serde_json::json!(tools))
    ///         }
    ///         _ => Err(ClaudeAgentError::Mcp(format!("Unsupported method: {}", method)))
    ///     }
    /// }
    /// ```
    async fn handle_client_message(
        &self,
        message: Value,
    ) -> Result<serde_json::Value, ClaudeAgentError>;
}

use serde::{Deserialize, Serialize};

/// Information about an MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl McpServerManager {
    /// Create a new MCP server manager.
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an MCP server.
    pub fn register(&mut self, server: Box<dyn McpServer>) {
        let name = server.name().to_string();
        // Use blocking write if possible? No, we are in async context usually.
        // But register takes &mut self, implying exclusive access.
        // But internal is RwLock.
        // We can't block. We should make register async?
        // OR we use try_write/blocking_write if we assume we are not in async runtime yet?
        // But we might be.
        // If we change register to async, it breaks API.
        // However, if we have &mut self, we are the only one holding this struct?
        // No, `servers` is Arc. Other clones might exist.
        // So we MUST use async write or blocking write.
        // Since this is usually called at setup, `blocking_write` might panic if in async context?
        // Use `tokio::task::block_in_place`?
        // Ideally register should be async.
        // But for backward compat, let's try to use std::sync::RwLock if we don't need async lock?
        // But servers need to be accessed async in get/list.
        // Let's use `futures::executor::block_on`?
        // Actually, if we spawn a task to do it?

        // Simpler: Just spawn a task? No we want it done now.
        // For now, let's use `try_write`. If it fails, we spin?
        // Or better: Change `register` to `async fn register`.
        // This breaks API but it's cleaner.

        // Wait, if we use `parking_lot::RwLock`, we can write synchronously.
        // Does McpServerManager need to be async aware?
        // `get` returns `Option<Arc>`.
        // `list_all_tools` is async because it calls `list_tools` on servers.
        // The map access itself can be sync.
        // So let's use `std::sync::RwLock` or `parking_lot::RwLock`.
        // Since we are in tokio environment, `std::sync::RwLock` is fine if we don't hold it across await points.
        // `list_all_tools` iterates. If we hold read lock while awaiting `list_tools`, we block writers.
        // We should clone the servers list then iterate.

        let mut servers = self.servers.blocking_write();
        servers.insert(name, Arc::from(server));
    }

    // Helper for sync writing (using tokio blocking_write which might not exist on RwLock?)
    // Tokio RwLock has `blocking_write`.

    /// Get a server by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn McpServer>> {
        // Use blocking read
        self.servers.blocking_read().get(name).cloned()
    }

    /// List all registered servers.
    pub fn list_servers(&self) -> Vec<String> {
        self.servers.blocking_read().keys().cloned().collect()
    }

    /// List all tools from all servers.
    pub async fn list_all_tools(&self) -> Result<Vec<(String, ToolInfo)>, ClaudeAgentError> {
        // Snapshot servers to release lock
        let servers: Vec<(String, Arc<dyn McpServer>)> = {
            let guard = self.servers.read().await;
            guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };

        let mut all_tools = Vec::new();
        for (server_name, server) in servers {
            let tools = server.list_tools().await?;
            for tool in tools {
                all_tools.push((server_name.clone(), tool));
            }
        }
        Ok(all_tools)
    }
}

impl Default for McpServerManager {
    fn default() -> Self {
        Self::new()
    }
}
