//! MCP SDK server for in-process tool hosting.
//!
//! [`SdkMcpServer`] manages tool registration and handles the JSON-RPC protocol
//! over stdio, making it compatible with the Claude CLI's MCP server spawning mechanism.
//!
//! # Lifecycle
//!
//! 1. Create a server with [`SdkMcpServer::new`]
//! 2. Register tools with [`SdkMcpServer::add_tool`] or [`SdkMcpServer::tool`]
//! 3. Run the server with [`SdkMcpServer::run_stdio`] to handle JSON-RPC over stdin/stdout

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::error::McpSdkError;
use crate::tool::{ToolDefinition, ToolHandler};

/// Configuration for an SDK-based MCP server, suitable for inclusion
/// in the main SDK's `mcp_servers` configuration.
///
/// Note: The actual [`SdkMcpServer`] instance is not serializable (it contains
/// trait objects for tool handlers). Use this config struct for metadata only,
/// and pass the server instance directly when running.
#[derive(Debug, Clone, Serialize)]
pub struct McpSdkServerConfig {
    /// Transport type. Always `"sdk"` for in-process servers.
    pub transport: String,
    /// Server name.
    pub server_name: String,
    /// Server version.
    pub server_version: String,
}

impl McpSdkServerConfig {
    /// Create a new config for an SDK-based MCP server.
    pub fn new(server: &SdkMcpServer) -> Self {
        Self {
            transport: "sdk".to_string(),
            server_name: server.name.clone(),
            server_version: server.version.clone(),
        }
    }
}

/// An in-process MCP server that communicates via JSON-RPC over stdio.
///
/// This server can be spawned by the Claude CLI as a subprocess. It handles
/// the MCP protocol messages (`initialize`, `tools/list`, `tools/call`) and
/// dispatches tool calls to registered handlers.
///
/// # Example
///
/// ```ignore
/// use claude_agent_mcp_sdk::server::SdkMcpServer;
/// use claude_agent_mcp_sdk::tool;
///
/// let mut server = SdkMcpServer::new("my-server", "0.1.0");
///
/// server.tool(
///     "greet",
///     "Greets the user",
///     serde_json::json!({
///         "type": "object",
///         "properties": {
///             "name": { "type": "string" }
///         },
///         "required": ["name"]
///     }),
///     |input| Box::pin(async move {
///         let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("World");
///         Ok(serde_json::json!({
///             "content": [{ "type": "text", "text": format!("Hello, {}!", name) }]
///         }))
///     }),
/// );
///
/// // Run the server (blocks until stdin closes)
/// // tokio::runtime::Runtime::new().unwrap().block_on(server.run_stdio());
/// ```
pub struct SdkMcpServer {
    /// Server name, exposed in MCP `serverInfo`.
    name: String,
    /// Server version, exposed in MCP `serverInfo`.
    version: String,
    /// Registered tools, keyed by name.
    tools: HashMap<String, ToolDefinition>,
}

impl SdkMcpServer {
    /// Create a new MCP SDK server.
    ///
    /// # Arguments
    ///
    /// * `name` - The server name sent to clients during initialization.
    /// * `version` - The server version string.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into(), tools: HashMap::new() }
    }

    /// Register a tool definition.
    ///
    /// If a tool with the same name already exists, it will be replaced.
    pub fn add_tool(&mut self, tool: ToolDefinition) {
        let name = tool.name.clone();
        self.tools.insert(name, tool);
    }

    /// Builder-style method for adding a tool.
    ///
    /// Returns `&mut Self` to allow chaining.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use claude_agent_mcp_sdk::server::SdkMcpServer;
    ///
    /// let mut server = SdkMcpServer::new("demo", "1.0.0");
    /// server
    ///     .tool("tool_a", "First tool", schema_a, handler_a)
    ///     .tool("tool_b", "Second tool", schema_b, handler_b);
    /// ```
    pub fn tool(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: impl ToolHandler + 'static,
    ) -> &mut Self {
        let def = ToolDefinition::new(name, description, input_schema, handler);
        self.add_tool(def);
        self
    }

    /// List the names of all registered tools.
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get a reference to a registered tool by name.
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Handle a single JSON-RPC request and return the response.
    ///
    /// This method is the core of the MCP protocol handling. It dispatches
    /// based on the `method` field of the request.
    pub async fn handle_request(&self, request: Value) -> Result<Value, McpSdkError> {
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = request.get("id").cloned();

        match method {
            "initialize" => self.handle_initialize(&id),
            "notifications/initialized" => {
                // Acknowledgment notification, no response needed
                Ok(Value::Null)
            },
            "tools/list" => self.handle_tools_list(&id),
            "tools/call" => self.handle_tools_call(&id, request).await,
            _ => Ok(jsonrpc_error(&id, -32601, &format!("Method not found: {method}"))),
        }
    }

    /// Run the server, reading JSON-RPC messages from stdin and writing
    /// responses to stdout.
    ///
    /// Each line on stdin should be a valid JSON-RPC request object.
    /// Each response is written as a single JSON line to stdout.
    ///
    /// This method blocks until stdin is closed or an unrecoverable error occurs.
    pub async fn run_stdio(&self) -> Result<(), McpSdkError> {
        let stdin = tokio::io::BufReader::new(tokio::io::stdin());
        let mut stdout = tokio::io::BufWriter::new(tokio::io::stdout());

        let mut lines = stdin.lines();

        while let Some(line) = lines.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let request: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) => {
                    let error_response = jsonrpc_error(&None, -32700, &format!("Parse error: {e}"));
                    write_json_response(&mut stdout, &error_response).await?;
                    continue;
                },
            };

            match self.handle_request(request).await {
                Ok(response) => {
                    if !response.is_null() {
                        write_json_response(&mut stdout, &response).await?;
                    }
                },
                Err(e) => {
                    let error_response =
                        jsonrpc_error(&None, -32000, &format!("Internal error: {e}"));
                    write_json_response(&mut stdout, &error_response).await?;
                },
            }
        }

        Ok(())
    }

    fn handle_initialize(&self, id: &Option<Value>) -> Result<Value, McpSdkError> {
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
                    "version": self.version
                }
            }
        }))
    }

    fn handle_tools_list(&self, id: &Option<Value>) -> Result<Value, McpSdkError> {
        let tools: Vec<Value> = self
            .tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect();

        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        }))
    }

    async fn handle_tools_call(
        &self,
        id: &Option<Value>,
        request: Value,
    ) -> Result<Value, McpSdkError> {
        let params = request
            .get("params")
            .ok_or_else(|| McpSdkError::InvalidInput("Missing params".to_string()))?;

        let tool_name = params
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| McpSdkError::InvalidInput("Missing tool name".to_string()))?;

        let arguments =
            params.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::new()));

        let tool = match self.tools.get(tool_name) {
            Some(t) => t,
            None => {
                return Ok(jsonrpc_error(
                    id,
                    -32000,
                    &McpSdkError::ToolNotFound(tool_name.to_string()).to_string(),
                ))
            },
        };

        match tool.handler.call(arguments).await {
            Ok(result) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            })),
            Err(e) => Ok(jsonrpc_error(id, -32000, &e.to_string())),
        }
    }
}

/// Write a JSON-RPC response as a single line to stdout.
async fn write_json_response(
    stdout: &mut tokio::io::BufWriter<tokio::io::Stdout>,
    response: &Value,
) -> Result<(), McpSdkError> {
    let response_bytes = format!("{}\n", serde_json::to_string(response)?);
    stdout.write_all(response_bytes.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}

/// Construct a JSON-RPC error response.
fn jsonrpc_error(id: &Option<Value>, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;

    struct EchoHandler;

    impl ToolHandler for EchoHandler {
        fn call(
            &self,
            input: Value,
        ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
            Box::pin(async move {
                let msg = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": msg }]
                }))
            })
        }
    }

    #[test]
    fn test_new_server() {
        let server = SdkMcpServer::new("test", "1.0.0");
        assert_eq!(server.tool_names().len(), 0);
    }

    #[test]
    fn test_add_tool() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        });

        server.add_tool(ToolDefinition::new(
            "echo",
            "Echoes a message",
            schema.clone(),
            EchoHandler,
        ));

        assert_eq!(server.tool_names(), vec!["echo"]);
        let tool = server.get_tool("echo").unwrap();
        assert_eq!(tool.name, "echo");
        assert_eq!(tool.description, "Echoes a message");
    }

    #[test]
    fn test_tool_builder() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({ "type": "object" });

        server.tool("echo", "Echoes a message", schema, EchoHandler).tool(
            "ping",
            "Returns pong",
            serde_json::json!({ "type": "object" }),
            EchoHandler,
        );

        assert_eq!(server.tool_names().len(), 2);
    }

    #[test]
    fn test_add_tool_replaces_existing() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({ "type": "object" });

        server.add_tool(ToolDefinition::new(
            "echo",
            "First description",
            schema.clone(),
            EchoHandler,
        ));
        server.add_tool(ToolDefinition::new("echo", "Second description", schema, EchoHandler));

        assert_eq!(server.tool_names().len(), 1);
        assert_eq!(server.get_tool("echo").unwrap().description, "Second description");
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let server = SdkMcpServer::new("my-server", "0.2.0");
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0.0" }
            }
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["result"]["serverInfo"]["name"], "my-server");
        assert_eq!(response["result"]["serverInfo"]["version"], "0.2.0");
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
        assert!(response["result"]["capabilities"]["tools"].is_object());
    }

    #[tokio::test]
    async fn test_handle_initialized_notification() {
        let server = SdkMcpServer::new("test", "1.0.0");
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let response = server.handle_request(request).await.unwrap();
        assert!(response.is_null());
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "message": { "type": "string" } }
        });

        server.add_tool(ToolDefinition::new("echo", "Echoes a message", schema, EchoHandler));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });

        let response = server.handle_request(request).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "echo");
        assert_eq!(tools[0]["description"], "Echoes a message");
        assert_eq!(tools[0]["inputSchema"]["type"], "object");
    }

    #[tokio::test]
    async fn test_handle_tools_call_success() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "message": { "type": "string" } }
        });

        server.add_tool(ToolDefinition::new("echo", "Echoes a message", schema, EchoHandler));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": { "message": "hello" }
            }
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["result"]["content"][0]["text"], "hello");
        assert_eq!(response["id"], 42);
    }

    #[tokio::test]
    async fn test_handle_tools_call_tool_not_found() {
        let server = SdkMcpServer::new("test", "1.0.0");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "nonexistent",
                "arguments": {}
            }
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["error"]["code"], -32000);
        assert!(response["error"]["message"].as_str().unwrap().contains("tool not found"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_params() {
        let server = SdkMcpServer::new("test", "1.0.0");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call"
        });

        let result = server.handle_request(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing params"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_tool_name() {
        let server = SdkMcpServer::new("test", "1.0.0");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "arguments": {}
            }
        });

        let result = server.handle_request(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Missing tool name"));
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let server = SdkMcpServer::new("test", "1.0.0");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "unknown/method"
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["error"]["code"], -32601);
        assert!(response["error"]["message"].as_str().unwrap().contains("Method not found"));
    }

    #[test]
    fn test_jsonrpc_error() {
        let err = jsonrpc_error(&Some(Value::Number(1.into())), -32602, "Invalid params");
        assert_eq!(err["jsonrpc"], "2.0");
        assert_eq!(err["id"], 1);
        assert_eq!(err["error"]["code"], -32602);
        assert_eq!(err["error"]["message"], "Invalid params");
    }

    #[test]
    fn test_config_serialization() {
        let server = SdkMcpServer::new("test", "1.0.0");
        let config = McpSdkServerConfig::new(&server);
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["transport"], "sdk");
        assert_eq!(json["server_name"], "test");
        assert_eq!(json["server_version"], "1.0.0");
    }

    #[tokio::test]
    async fn test_tools_call_empty_arguments() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({ "type": "object" });

        server.add_tool(ToolDefinition::new("echo", "Echo", schema, EchoHandler));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "echo"
            }
        });

        let response = server.handle_request(request).await.unwrap();
        // Empty message defaults to empty string
        assert_eq!(response["result"]["content"][0]["text"], "");
    }

    #[tokio::test]
    async fn test_initialize_preserves_request_id() {
        let server = SdkMcpServer::new("test", "1.0.0");

        // Test with numeric ID
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0.0" }
            }
        });
        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["id"], 99);

        // Test with string ID
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0.0" }
            }
        });
        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["id"], "req-1");
    }

    #[tokio::test]
    async fn test_tools_list_preserves_request_id() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({ "type": "object" });
        server.add_tool(ToolDefinition::new("echo", "Echo", schema, EchoHandler));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "list-1",
            "method": "tools/list"
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["id"], "list-1");
    }

    #[tokio::test]
    async fn test_tools_call_preserves_request_id() {
        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({ "type": "object" });
        server.add_tool(ToolDefinition::new("echo", "Echo", schema, EchoHandler));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call-abc",
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": { "message": "test" }
            }
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["id"], "call-abc");
    }

    #[tokio::test]
    async fn test_tools_call_handler_error_returns_jsonrpc_error() {
        struct ErrorHandler;
        impl ToolHandler for ErrorHandler {
            fn call(
                &self,
                _input: Value,
            ) -> Pin<Box<dyn Future<Output = Result<Value, McpSdkError>> + Send + '_>> {
                Box::pin(async {
                    Err(McpSdkError::HandlerError("something went wrong".to_string()))
                })
            }
        }

        let mut server = SdkMcpServer::new("test", "1.0.0");
        let schema = serde_json::json!({ "type": "object" });
        server.add_tool(ToolDefinition::new("fail", "Always fails", schema, ErrorHandler));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "fail",
                "arguments": {}
            }
        });

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response["error"]["code"], -32000);
        assert!(response["error"]["message"].as_str().unwrap().contains("something went wrong"));
    }

    #[tokio::test]
    async fn test_tools_list_empty_server() {
        let server = SdkMcpServer::new("empty", "1.0.0");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });

        let response = server.handle_request(request).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_tools_list_multiple_tools() {
        let mut server = SdkMcpServer::new("test", "1.0.0");

        server.add_tool(ToolDefinition::new(
            "tool_a",
            "First tool",
            serde_json::json!({ "type": "object" }),
            EchoHandler,
        ));
        server.add_tool(ToolDefinition::new(
            "tool_b",
            "Second tool",
            serde_json::json!({ "type": "object" }),
            EchoHandler,
        ));
        server.add_tool(ToolDefinition::new(
            "tool_c",
            "Third tool",
            serde_json::json!({ "type": "object" }),
            EchoHandler,
        ));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });

        let response = server.handle_request(request).await.unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn test_get_tool_nonexistent() {
        let server = SdkMcpServer::new("test", "1.0.0");
        assert!(server.get_tool("nonexistent").is_none());
    }
}
