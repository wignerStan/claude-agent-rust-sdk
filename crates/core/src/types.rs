//! Response types for agent control methods.
//!
//! These types are defined locally within `claude-agent-core` rather than in
//! `claude-agent-types` to keep the dependency boundary clean.

use serde::{Deserialize, Serialize};

/// Status of an MCP server connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpConnectionStatus {
    /// Server is connected and operational.
    Connected,
    /// Server is currently connecting.
    Pending,
    /// Server connection failed.
    Failed,
    /// Server requires authentication.
    NeedsAuth,
    /// Server is disabled.
    Disabled,
}

impl std::fmt::Display for McpConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected => write!(f, "connected"),
            Self::Pending => write!(f, "pending"),
            Self::Failed => write!(f, "failed"),
            Self::NeedsAuth => write!(f, "needs-auth"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// Tool information from an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    /// Tool name.
    pub name: String,
    /// Human-readable description of the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Status information for a single MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerStatus {
    /// Server name as configured.
    pub name: String,
    /// Current connection status.
    pub status: McpConnectionStatus,
    /// Server information from MCP handshake (available when connected).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_info: Option<serde_json::Value>,
    /// Error message (available when status is `Failed`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Server configuration details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Configuration scope (e.g., project, user, local).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Tools provided by this server (available when connected).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<McpToolInfo>,
}

/// Response from `ClaudeAgent::get_mcp_status()`.
///
/// Wraps the list of server statuses under the `mcp_servers` key,
/// matching the wire-format response shape from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStatusResponse {
    /// List of MCP server statuses.
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: Vec<McpServerStatus>,
}

impl McpStatusResponse {
    /// Create an empty MCP status response.
    pub fn empty() -> Self {
        Self { mcp_servers: Vec::new() }
    }
}

impl Default for McpStatusResponse {
    fn default() -> Self {
        Self::empty()
    }
}

/// A single context usage category (system prompt, tools, messages, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsageCategory {
    /// Category name.
    pub name: String,
    /// Token count for this category.
    pub tokens: u64,
    /// Display color for UI rendering.
    #[serde(default)]
    pub color: String,
    /// Whether this category is deferred (loaded lazily).
    #[serde(default, rename = "isDeferred", skip_serializing_if = "Option::is_none")]
    pub is_deferred: Option<bool>,
}

/// Response from `ClaudeAgent::get_context_usage()`.
///
/// Provides a breakdown of current context window usage by category,
/// matching the data shown by the `/context` command in the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUsageResponse {
    /// Token usage broken down by category.
    #[serde(default)]
    pub categories: Vec<ContextUsageCategory>,
    /// Total tokens currently in the context window.
    #[serde(rename = "totalTokens", default)]
    pub total_tokens: u64,
    /// Effective maximum tokens (may be reduced by autocompact buffer).
    #[serde(rename = "maxTokens", default)]
    pub max_tokens: u64,
    /// Raw model context window size.
    #[serde(rename = "rawMaxTokens", default)]
    pub raw_max_tokens: u64,
    /// Percentage of context window used (0-100).
    #[serde(default)]
    pub percentage: f64,
    /// Model name the context usage is calculated for.
    #[serde(default)]
    pub model: String,
    /// Whether autocompact is enabled for this session.
    #[serde(rename = "isAutoCompactEnabled", default)]
    pub is_auto_compact_enabled: bool,
    /// CLAUDE.md and memory files loaded.
    #[serde(rename = "memoryFiles", default, skip_serializing_if = "Vec::is_empty")]
    pub memory_files: Vec<serde_json::Value>,
    /// MCP tools with name, serverName, tokens, and isLoaded status.
    #[serde(rename = "mcpTools", default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_tools: Vec<serde_json::Value>,
    /// Agent definitions with agentType, source, and token counts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<serde_json::Value>,
    /// Visual grid representation used by the CLI context display.
    #[serde(rename = "gridRows", default, skip_serializing_if = "Vec::is_empty")]
    pub grid_rows: Vec<Vec<serde_json::Value>>,
}

impl Default for ContextUsageResponse {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            total_tokens: 0,
            max_tokens: 0,
            raw_max_tokens: 0,
            percentage: 0.0,
            model: String::new(),
            is_auto_compact_enabled: false,
            memory_files: Vec::new(),
            mcp_tools: Vec::new(),
            agents: Vec::new(),
            grid_rows: Vec::new(),
        }
    }
}

/// Server initialization information from Claude Code CLI.
///
/// Returned by `ClaudeAgent::get_server_info()`. Contains metadata about
/// the connected Claude Code server, including available commands,
/// output styles, and server capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Raw initialization data from the server.
    ///
    /// This preserves the full JSON structure from the CLI's init message,
    /// which may include `commands`, `output_style`, and other fields that
    /// vary by CLI version.
    pub data: serde_json::Value,
}

impl ServerInfo {
    /// Create server info from raw JSON data.
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }

    /// Get the output style, if present.
    pub fn output_style(&self) -> Option<&str> {
        self.data.get("output_style").and_then(|v| v.as_str())
    }

    /// Get the list of available commands, if present.
    pub fn commands(&self) -> Option<Vec<&str>> {
        self.data.get("commands").and_then(|v| {
            v.as_array().map(|arr| arr.iter().filter_map(|item| item.as_str()).collect())
        })
    }

    /// Get a specific field from the server info data.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_status_response_deserialization() {
        let json = r#"{"mcpServers":[{"name":"test-server","status":"connected","tools":[{"name":"tool1","description":"A tool"}]}]}"#;
        let response: McpStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.mcp_servers.len(), 1);
        assert_eq!(response.mcp_servers[0].name, "test-server");
        assert_eq!(response.mcp_servers[0].status, McpConnectionStatus::Connected);
        assert_eq!(response.mcp_servers[0].tools.len(), 1);
        assert_eq!(response.mcp_servers[0].tools[0].name, "tool1");
    }

    #[test]
    fn mcp_status_response_empty() {
        let json = r#"{"mcpServers":[]}"#;
        let response: McpStatusResponse = serde_json::from_str(json).unwrap();
        assert!(response.mcp_servers.is_empty());
    }

    #[test]
    fn mcp_status_response_with_error() {
        let json = r#"{"mcpServers":[{"name":"bad-server","status":"failed","error":"Connection refused"}]}"#;
        let response: McpStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.mcp_servers[0].status, McpConnectionStatus::Failed);
        assert_eq!(response.mcp_servers[0].error.as_deref(), Some("Connection refused"));
    }

    #[test]
    fn mcp_connection_status_display() {
        assert_eq!(McpConnectionStatus::Connected.to_string(), "connected");
        assert_eq!(McpConnectionStatus::Failed.to_string(), "failed");
        assert_eq!(McpConnectionStatus::NeedsAuth.to_string(), "needs-auth");
    }

    #[test]
    fn context_usage_response_deserialization() {
        let json = r#"{
            "categories":[{"name":"system","tokens":500,"color":"blue"}],
            "totalTokens":2000,
            "maxTokens":200000,
            "rawMaxTokens":200000,
            "percentage":1.0,
            "model":"claude-sonnet-4-5",
            "isAutoCompactEnabled":true,
            "memoryFiles":[],
            "mcpTools":[],
            "agents":[],
            "gridRows":[]
        }"#;
        let response: ContextUsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.categories.len(), 1);
        assert_eq!(response.categories[0].name, "system");
        assert_eq!(response.categories[0].tokens, 500);
        assert_eq!(response.total_tokens, 2000);
        assert_eq!(response.percentage, 1.0);
        assert!(response.is_auto_compact_enabled);
    }

    #[test]
    fn context_usage_response_default() {
        let response = ContextUsageResponse::default();
        assert!(response.categories.is_empty());
        assert_eq!(response.total_tokens, 0);
        assert_eq!(response.percentage, 0.0);
    }

    #[test]
    fn server_info_construction() {
        let data = serde_json::json!({
            "output_style": "concise",
            "commands": ["/help", "/context"]
        });
        let info = ServerInfo::new(data);
        assert_eq!(info.output_style(), Some("concise"));
        let cmds = info.commands().unwrap();
        assert_eq!(cmds, vec!["/help", "/context"]);
    }

    #[test]
    fn server_info_get_field() {
        let data = serde_json::json!({"custom_key": "custom_value"});
        let info = ServerInfo::new(data);
        assert_eq!(info.get("custom_key").and_then(|v| v.as_str()), Some("custom_value"));
        assert!(info.get("missing").is_none());
    }

    #[test]
    fn mcp_status_response_serialization_roundtrip() {
        let response = McpStatusResponse {
            mcp_servers: vec![McpServerStatus {
                name: "my-server".to_string(),
                status: McpConnectionStatus::Connected,
                server_info: None,
                error: None,
                config: None,
                scope: Some("project".to_string()),
                tools: vec![],
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: McpStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mcp_servers.len(), 1);
        assert_eq!(parsed.mcp_servers[0].name, "my-server");
        assert_eq!(parsed.mcp_servers[0].scope.as_deref(), Some("project"));
    }
}
