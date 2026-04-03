//! MCP transport clients using rmcp for protocol handling.
//!
//! Provides MCP client connections to remote servers via:
//! - **StdioMcpServer**: Subprocess communication over stdio
//! - **HttpMcpServer**: HTTP-based JSON-RPC (streamable HTTP)
//! - **SseMcpServer**: SSE-based JSON-RPC (same transport, kept for API compat)

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::OnceCell;

use rmcp::model::CallToolRequestParams;
use rmcp::service::{Peer, RunningService, ServiceExt};
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::RoleClient;

use crate::mcp::manager::{McpServer, ToolInfo};
use crate::types::ClaudeAgentError;

/// Convert rmcp Tool to our ToolInfo.
impl From<rmcp::model::Tool> for ToolInfo {
    fn from(tool: rmcp::model::Tool) -> Self {
        ToolInfo {
            name: tool.name.into_owned(),
            description: tool.description.map(|d| d.into_owned()),
            input_schema: serde_json::to_value(&*tool.input_schema).unwrap_or_default(),
        }
    }
}

/// Stdio-based MCP client — connects to a subprocess via rmcp transport.
pub struct StdioMcpServer {
    name: String,
    command: String,
    args: Vec<String>,
    peer: OnceCell<Peer<RoleClient>>,
}

impl StdioMcpServer {
    /// Create a new stdio MCP client.
    pub fn new(name: String, command: String, args: Vec<String>) -> Result<Self, ClaudeAgentError> {
        Ok(Self { name, command, args, peer: OnceCell::new() })
    }

    async fn ensure_connected(&self) -> Result<&Peer<RoleClient>, ClaudeAgentError> {
        self.peer
            .get_or_try_init(|| async {
                let mut cmd = tokio::process::Command::new(&self.command);
                cmd.args(&self.args);
                let transport = TokioChildProcess::new(cmd).map_err(|e| {
                    ClaudeAgentError::Mcp(format!("Failed to spawn {}: {}", self.name, e))
                })?;
                let running: RunningService<RoleClient, ()> =
                    ().serve(transport).await.map_err(|e| {
                        ClaudeAgentError::Mcp(format!(
                            "MCP handshake failed for {}: {:?}",
                            self.name, e
                        ))
                    })?;
                let peer = running.peer().clone();
                // Detach the background task — it keeps running as long as transport is alive
                tokio::spawn(async move {
                    let _ = running;
                });
                Ok(peer)
            })
            .await
    }
}

#[async_trait]
impl McpServer for StdioMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        let peer = self.ensure_connected().await?;
        let tools = peer
            .list_all_tools()
            .await
            .map_err(|e| ClaudeAgentError::Mcp(format!("list_tools failed: {:?}", e)))?;
        Ok(tools.into_iter().map(ToolInfo::from).collect())
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, ClaudeAgentError> {
        let peer = self.ensure_connected().await?;
        let params = CallToolRequestParams::new(name.to_string())
            .with_arguments(serde_json::from_value(arguments).unwrap_or_default());
        let result = peer
            .call_tool(params)
            .await
            .map_err(|e| ClaudeAgentError::Mcp(format!("call_tool failed: {:?}", e)))?;
        Ok(serde_json::to_value(result).unwrap_or_default())
    }
}

/// HTTP-based MCP client using rmcp's streamable HTTP transport.
pub struct HttpMcpServer {
    name: String,
    url: String,
    peer: OnceCell<Peer<RoleClient>>,
}

impl HttpMcpServer {
    /// Create a new HTTP MCP client.
    pub fn new(name: String, url: String) -> Result<Self, ClaudeAgentError> {
        Ok(Self { name, url, peer: OnceCell::new() })
    }

    /// Create with timeout (kept for API compat; rmcp manages timeouts internally).
    pub fn with_timeout(
        name: String,
        url: String,
        _timeout: std::time::Duration,
    ) -> Result<Self, ClaudeAgentError> {
        Self::new(name, url)
    }

    async fn ensure_connected(&self) -> Result<&Peer<RoleClient>, ClaudeAgentError> {
        self.peer
            .get_or_try_init(|| async {
                let transport =
                    rmcp::transport::StreamableHttpClientTransport::from_uri(self.url.clone());
                let running: RunningService<RoleClient, ()> =
                    ().serve(transport).await.map_err(|e| {
                        ClaudeAgentError::Mcp(format!(
                            "HTTP MCP handshake failed for {}: {:?}",
                            self.name, e
                        ))
                    })?;
                let peer = running.peer().clone();
                tokio::spawn(async move {
                    let _ = running;
                });
                Ok(peer)
            })
            .await
    }
}

#[async_trait]
impl McpServer for HttpMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        let peer = self.ensure_connected().await?;
        let tools = peer
            .list_all_tools()
            .await
            .map_err(|e| ClaudeAgentError::Mcp(format!("list_tools failed: {:?}", e)))?;
        Ok(tools.into_iter().map(ToolInfo::from).collect())
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, ClaudeAgentError> {
        let peer = self.ensure_connected().await?;
        let params = CallToolRequestParams::new(name.to_string())
            .with_arguments(serde_json::from_value(arguments).unwrap_or_default());
        let result = peer
            .call_tool(params)
            .await
            .map_err(|e| ClaudeAgentError::Mcp(format!("call_tool failed: {:?}", e)))?;
        Ok(serde_json::to_value(result).unwrap_or_default())
    }
}

/// SSE-based MCP client (uses same HTTP transport; kept for API compat).
pub struct SseMcpServer {
    inner: HttpMcpServer,
}

impl SseMcpServer {
    /// Create a new SSE MCP client.
    pub fn new(name: String, url: String) -> Result<Self, ClaudeAgentError> {
        Ok(Self { inner: HttpMcpServer::new(name, url)? })
    }

    /// Create with timeout.
    pub fn with_timeout(
        name: String,
        url: String,
        timeout: std::time::Duration,
    ) -> Result<Self, ClaudeAgentError> {
        Ok(Self { inner: HttpMcpServer::with_timeout(name, url, timeout)? })
    }
}

#[async_trait]
impl McpServer for SseMcpServer {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClaudeAgentError> {
        self.inner.list_tools().await
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, ClaudeAgentError> {
        self.inner.call_tool(name, arguments).await
    }
}
