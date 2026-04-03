use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use serde_json::Value;

use crate::mcp::manager::{McpServer, ToolInfo};
use crate::types::ClaudeAgentError;

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

    // handle_client_message uses the default implementation from the trait
}
