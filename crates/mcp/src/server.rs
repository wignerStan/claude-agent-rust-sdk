use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use serde_json::Value;

use crate::manager::{McpServer, ToolInfo};
use claude_agent_types::ClaudeAgentError;

/// Type alias for async tool handler.
pub type ToolHandler = Box<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ClaudeAgentError>> + Send>>
        + Send
        + Sync,
>;

/// SDK-hosted MCP server.
pub struct SdkMcpServer {
    name: String,
    tools: HashMap<String, (ToolInfo, ToolHandler)>,
}

impl SdkMcpServer {
    /// Create new SDK server.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), tools: HashMap::new() }
    }

    /// Register a tool.
    pub fn register_tool<F, Fut>(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
        input_schema: Value,
        handler: F,
    ) where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, ClaudeAgentError>> + Send + 'static,
    {
        let name = name.into();
        let info = ToolInfo { name: name.clone(), description, input_schema };

        // Box the handler to handle generic Future return type
        let boxed_handler = Box::new(move |args| {
            let fut = handler(args);
            Box::pin(fut) as Pin<Box<dyn Future<Output = _> + Send>>
        });

        self.tools.insert(name, (info, boxed_handler));
    }
}

#[async_trait]
impl McpServer for SdkMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        Ok(self.tools.values().map(|(info, _)| info.clone()).collect())
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, ClaudeAgentError> {
        if let Some((_, handler)) = self.tools.get(name) {
            handler(arguments).await
        } else {
            Err(ClaudeAgentError::Mcp(format!("Tool not found: {}", name)))
        }
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
            },
            Some("tools/list") => {
                let tools = self.list_tools().await?;
                // Convert ToolInfo to schema expected by JSON-RPC (which might just be list of tools)
                // Python SDK returns: { result: { tools: [...] } }
                // ToolInfo serializes to expected format.
                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools
                    }
                }))
            },
            Some("tools/call") => {
                if let Some(p) = params {
                    let name = p.get("name").and_then(|n| n.as_str());
                    let args = p.get("arguments").cloned().unwrap_or(serde_json::json!({}));

                    if let Some(tool_name) = name {
                        match self.call_tool(tool_name, args).await {
                            Ok(result) => {
                                // Convert tool result (which is just content?) to MCP format?
                                // Python SDK: returns { content: [...] }
                                // Handler returns { content: [...] } usually.
                                // We wrap it in result.
                                Ok(serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": result
                                }))
                            },
                            Err(e) => {
                                // Return JSON-RPC error inside 200 OK wrapper if possible?
                                // Or just error field?
                                Ok(serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": {
                                        "code": -32000,
                                        "message": e.to_string()
                                    }
                                }))
                            },
                        }
                    } else {
                        Ok(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32602, "message": "Missing tool name" }
                        }))
                    }
                } else {
                    Ok(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32602, "message": "Missing params" }
                    }))
                }
            },
            Some("notifications/initialized") => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "result": {}
            })),
            _ => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {:?}", method) }
            })),
        }
    }
}
