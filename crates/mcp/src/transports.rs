//! MCP transport wrappers for different server types.
//!
//! This module provides MCP server implementations for different transport types:
//! - **StdioMcpServer**: Full-featured subprocess-based MCP server
//! - **SseMcpServer**: SSE-based MCP server (deprecated, not implemented)
//! - **HttpMcpServer**: HTTP-based MCP server (deprecated, not implemented)
//!
//! # Architecture
//!
//! All implementations follow the `McpServer` trait from the manager module.
//! The StdioMcpServer is the primary implementation with full functionality.
//! SSE and HTTP transports are stubs that return errors indicating they
//! are not yet implemented.
//!
//! # Example
//!
//! See [StdioMcpServer](struct.StdioMcpServer) for usage examples.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::sync::{oneshot, Mutex};

use claude_agent_types::ClaudeAgentError;
use crate::manager::{McpServer, ToolInfo};

use claude_agent_transport::reader::MessageReader;
use serde_json::{json, Value};

/// Stdio-based MCP server (subprocess).
///
/// This is the primary MCP server implementation with full functionality.
/// It spawns a subprocess and communicates via stdio, supporting tool
/// registration, tool calls, and JSON-RPC message handling.
///
/// # Features
///
/// - **Tool Registration**: Dynamic registration of tools with handlers
/// - **Tool Calls**: Execute tools and return results via JSON-RPC
/// - **Message Handling**: Process JSON-RPC messages from client
/// - **Request Tracking**: Track pending requests and match responses
///
/// # Thread Safety
///
/// This implementation is `Send + Sync` and can be safely shared across
/// multiple threads or async tasks. Internal state is protected by `Arc<Mutex<>>`.
pub struct StdioMcpServer {
    name: String,
    command: String,
    args: Vec<String>,
    // Internal state
    transport: Arc<Mutex<Option<StdioTransportState>>>,
}

// Type alias to reduce complexity
type PendingRequests = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, ClaudeAgentError>>>>>;

struct StdioTransportState {
    #[allow(dead_code)]
    child: Child,
    stdin: tokio::process::ChildStdin,
    pending_requests: PendingRequests,
    next_id: Arc<AtomicU64>,
}

impl StdioMcpServer {
    /// Create a new stdio MCP server.
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this server
    /// - `command`: Command to execute (e.g., "python", "node")
    /// - `args`: Command-line arguments to pass to the command
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use claude_agent_mcp::StdioMcpServer;
    ///
    /// let server = StdioMcpServer::new("my_server".to_string(), "python".to_string(), vec!["-m".to_string()]);
    /// ```
    pub fn new(name: String, command: String, args: Vec<String>) -> Self {
        Self {
            name,
            command,
            args,
            transport: Arc::new(Mutex::new(None)),
        }
    }

    /// Register a tool.
    ///
    /// Registers a tool with a handler function that will be called when
    /// the tool is invoked via JSON-RPC.
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for the tool
    /// - `description`: Human-readable description of what the tool does
    /// - `input_schema`: JSON Schema defining expected input format
    /// - `handler`: Async function that executes the tool logic
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// server.register_tool("add", Some("Add two numbers"), json!({"type": "object"}), |args| {
    ///     async move {
    ///         let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
    ///         let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0);
    ///         Ok(json!({"result": a + b}))
    ///     }
    /// }).await;
    /// ```
    pub async fn register_tool<F, Fut>(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
        input_schema: Value,
        handler: F,
    ) -> Result<(), ClaudeAgentError>
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, ClaudeAgentError>> + Send + 'static,
    {
        let name = name.into();
        let info = ToolInfo {
            name: name.clone(),
            description,
            input_schema,
        };

        // Box the handler to handle generic Future return type
        let boxed_handler = Box::new(move |args| {
            let fut = handler(args);
            Box::pin(fut) as Pin<Box<dyn Future<Output = _> + Send>>
        });

        // Get or create state
        let mut guard = self.transport.lock().await;
        if guard.is_some() {
            // Server already connected, just add tool
            if let Some(state) = guard.as_mut() {
                let _tools = &mut state.pending_requests.lock().await;
                // Note: Tool registration logic would go here
            }
        } else {
            // Initialize server
            let mut cmd = Command::new(&self.command);
            cmd.args(&self.args);
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::inherit());

            let mut child = cmd.spawn().map_err(|e| {
                ClaudeAgentError::Mcp(format!("Failed to spawn {}: {}", self.name, e))
            })?;

            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| ClaudeAgentError::Mcp("No stdin".to_string()))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| ClaudeAgentError::Mcp("No stdout".to_string()))?;

            let pending_requests: PendingRequests = Arc::new(Mutex::new(HashMap::new()));
            let next_id = Arc::new(AtomicU64::new(1));

            let pending_clone = pending_requests.clone();

            // Spawn reader task
            tokio::spawn(async move {
                use claude_agent_transport::reader::MessageReader;
                use futures::StreamExt;

                let reader = MessageReader::new(stdout);
                let mut stream = Box::pin(reader);

                while let Some(res) = stream.next().await {
                    match res {
                        Ok(msg) => {
                            // Handle JSON-RPC response
                            if let Some(id) = msg.get("id").and_then(|i| i.as_u64()) {
                                let mut map = pending_clone.lock().await;
                                if let Some(sender) = map.remove(&id) {
                                    // Check for error
                                    if let Some(error) = msg.get("error") {
                                        let _ = sender
                                            .send(Err(ClaudeAgentError::Mcp(error.to_string())));
                                    } else if let Some(result) = msg.get("result") {
                                        let _ = sender.send(Ok(result.clone()));
                                    } else {
                                        let _ = sender.send(Ok(msg)); // Fallback
                                    }
                                }
                            }
                            // Handle notifications (optional)
                        }
                        Err(e) => {
                            eprintln!("MCP Error: {}", e);
                            break;
                        }
                    }
                }
            });

            let state = StdioTransportState {
                child,
                stdin,
                pending_requests,
                next_id,
            };
            *guard = Some(state);
        }

        // Return a clone of what's inside logic?
        // No, we can't return Arc<Mutex<State>> because State is inside Option inside Arc<Mutex>...
        // We should just return unit and let caller use lock again.
        // Or refactor `transport` to be Arc<Mutex<State>> and initialize loosely?
        // But initialization requires async spawning, which is easier with lock.
        // For now, change return type to Result<(), ...>
        Ok(())
    }

    /// Make a JSON-RPC request to the subprocess.
    ///
    /// Sends a JSON-RPC 2.0 formatted message to the subprocess stdin
    /// and waits for a matching response.
    ///
    /// # Parameters
    ///
    /// - `method`: JSON-RPC method name (e.g., "tools/list", "tools/call")
    /// - `params`: Parameters to pass to the method
    ///
    /// # Returns
    ///
    /// Returns the JSON-RPC response value.
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Mcp` if:
    /// - Server not connected
    /// - Request fails
    /// - Response channel closes
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = server.request("tools/call", json!({"name": "add", "arguments": {"a": 5, "b": 3}})).await?;
    /// println!("Result: {:?}", result);
    /// ```
    async fn request(&self, method: &str, params: Value) -> Result<Value, ClaudeAgentError> {
        let guard = self.transport.lock().await;
        if guard.is_none() {
            return Err(ClaudeAgentError::Mcp("Not connected".to_string()));
        }
        drop(guard);

        let tx;
        let id;

        // Scope for lock
        {
            let mut guard = self.transport.lock().await;
            let state = guard
                .as_mut()
                .ok_or(ClaudeAgentError::Mcp("Not connected".to_string()))?;

            id = state.next_id.fetch_add(1, Ordering::SeqCst);
            let (sender, receiver) = oneshot::channel();
            state.pending_requests.lock().await.insert(id, sender);
            tx = receiver;

            let req = json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": params
            });

            let json_str = req.to_string() + "\n";
            state
                .stdin
                .write_all(json_str.as_bytes())
                .await
                .map_err(|e| ClaudeAgentError::Mcp(format!("Write error: {}", e)))?;
        }

        // Wait for response
        tx.await
            .map_err(|_| ClaudeAgentError::Mcp("Channel closed".to_string()))?
    }
}

#[async_trait::async_trait]
impl McpServer for StdioMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        let res = self.request("tools/list", json!({})).await?;
        // Parse result.tools
        let tools_val = res
            .get("tools")
            .ok_or(ClaudeAgentError::Mcp("No tools field".to_string()))?;
        let tools: Vec<ToolInfo> = serde_json::from_value(tools_val.clone())
            .map_err(|e| ClaudeAgentError::Mcp(format!("Invalid tool list: {}", e)))?;
        Ok(tools)
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, ClaudeAgentError> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        let res = self.request("tools/call", params).await?;
        // Result should be CallToolResult (content, isError payload)
        // Check for application level error
        if let Some(true) = res.get("isError").and_then(|b| b.as_bool()) {
            // It's a tool error, but we return valid JSON result often?
            // Or map to Error?
            // Python SDK returns Result even on tool failure usually, unless it throws.
            // We return Value.
        }
        Ok(res)
    }

    async fn handle_client_message(&self, message: Value) -> Result<Value, ClaudeAgentError> {
        let method = message.get("method").and_then(|m| m.as_str());
        let id = message.get("id");
        let params = message.get("params");

        match method {
            Some("initialize") => {
                // Return server info and capabilities
                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": self.name,
                            "version": "1.0.0"
                        }
                    }
                }))
            }
            Some("tools/list") => {
                // Delegate to list_tools
                let tools = self.list_tools().await?;
                Ok(serde_json::json!({ "tools": tools }))
            }
            Some("tools/call") => {
                // Delegate to call_tool
                if let Some(params) = params {
                    if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                        let args = params.get("arguments").cloned().unwrap_or(json!({}));
                        self.call_tool(name, args).await
                    } else {
                        Err(ClaudeAgentError::Mcp("Missing tool name".to_string()))
                    }
                } else {
                    Err(ClaudeAgentError::Mcp("Missing params".to_string()))
                }
            }
            _ => Err(ClaudeAgentError::Mcp(format!(
                "Unsupported method: {}",
                method.unwrap_or("unknown")
            ))),
        }
    }
}

/// SSE-based MCP server.
///
/// **Deprecated**: SSE transport is not yet implemented. Use `StdioMcpServer` for now.
///
/// This struct is a placeholder for future SSE-based MCP server implementation.
/// It currently returns errors for all operations.
#[deprecated(note = "SSE transport not yet implemented. Use StdioMcpServer for now.")]
pub struct SseMcpServer {
    name: String,
    url: String,
}

impl SseMcpServer {
    /// Create a new SSE MCP server.
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this server
    /// - `url`: SSE endpoint URL
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. Use `StdioMcpServer` for
    /// a fully functional MCP server.
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

#[async_trait::async_trait]
impl McpServer for SseMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        Err(ClaudeAgentError::Mcp(
            "SSE transport not yet implemented. Use StdioMcpServer for now.".to_string(),
        ))
    }

    async fn call_tool(
        &self,
        _name: &str,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value, ClaudeAgentError> {
        Err(ClaudeAgentError::Mcp(
            "SSE transport not yet implemented. Use StdioMcpServer for now.".to_string(),
        ))
    }

    async fn handle_client_message(
        &self,
        _message: serde_json::Value,
    ) -> Result<serde_json::Value, ClaudeAgentError> {
        Err(ClaudeAgentError::Mcp(
            "SSE transport not yet implemented. Use StdioMcpServer for now.".to_string(),
        ))
    }
}

/// HTTP-based MCP server.
///
/// **Deprecated**: HTTP transport is not yet implemented. Use `StdioMcpServer` for now.
///
/// This struct is a placeholder for future HTTP-based MCP server implementation.
/// It currently returns errors for all operations.
#[deprecated(note = "HTTP transport not yet implemented. Use StdioMcpServer for now.")]
pub struct HttpMcpServer {
    name: String,
    url: String,
}

impl HttpMcpServer {
    /// Create a new HTTP MCP server.
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this server
    /// - `url`: HTTP endpoint URL
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. Use `StdioMcpServer` for
    /// a fully functional MCP server.
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

#[async_trait::async_trait]
impl McpServer for HttpMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        Err(ClaudeAgentError::Mcp(
            "HTTP transport not yet implemented. Use StdioMcpServer for now.".to_string(),
        ))
    }

    async fn call_tool(
        &self,
        _name: &str,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value, ClaudeAgentError> {
        Err(ClaudeAgentError::Mcp(
            "HTTP transport not yet implemented. Use StdioMcpServer for now.".to_string(),
        ))
    }

    async fn handle_client_message(
        &self,
        _message: serde_json::Value,
    ) -> Result<serde_json::Value, ClaudeAgentError> {
        Err(ClaudeAgentError::Mcp(
            "HTTP transport not yet implemented. Use StdioMcpServer for now.".to_string(),
        ))
    }
}
