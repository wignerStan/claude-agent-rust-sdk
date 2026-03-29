use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
// Hook types are handled via callbacks in Rust

/// Permission mode controlling how Claude interacts with tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Default permission mode with standard prompts.
    Default,
    /// Automatically accept file edits.
    AcceptEdits,
    /// Plan-only mode with no tool execution.
    Plan,
    /// Bypass all permission checks.
    BypassPermissions,
    /// Never ask for permission; auto-accept or auto-deny.
    DontAsk,
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::AcceptEdits => write!(f, "acceptEdits"),
            Self::Plan => write!(f, "plan"),
            Self::BypassPermissions => write!(f, "bypassPermissions"),
            Self::DontAsk => write!(f, "dontAsk"),
        }
    }
}

/// Transport type for MCP server connections.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum McpTransportType {
    /// Use stdio-based transport (subprocess)
    #[default]
    Stdio,
    /// Use HTTP-based transport
    Http,
    /// Use SSE-based transport
    Sse,
    /// Auto-detect transport (try HTTP, fallback to stdio)
    Auto,
}

impl std::fmt::Display for McpTransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio => write!(f, "stdio"),
            Self::Http => write!(f, "http"),
            Self::Sse => write!(f, "sse"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

/// Level of effort to use for Claude's responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EffortLevel {
    /// Minimal effort for simple tasks.
    Low,
    /// Standard effort for most tasks.
    Medium,
    /// Increased effort for complex tasks.
    High,
    /// Maximum effort for the most demanding tasks.
    Max,
}

impl std::fmt::Display for EffortLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Max => write!(f, "max"),
        }
    }
}

/// Configuration for an MCP server connection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    /// Transport type to use
    #[serde(default)]
    pub transport: McpTransportType,
    /// Command to execute (for stdio transport)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Arguments for the command (for stdio transport)
    #[serde(default)]
    pub args: Vec<String>,
    /// URL for HTTP/SSE transport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Request timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Environment variables for subprocess
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SettingSource {
    User,
    Project,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum SystemPromptPreset {
    #[serde(rename = "preset")]
    Preset {
        preset: String, // e.g., "claude_code"
        #[serde(skip_serializing_if = "Option::is_none")]
        append: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ToolsPreset {
    #[serde(rename = "preset")]
    Preset {
        preset: String, // e.g., "claude_code"
    },
}

/// Scope for memory storage across sessions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum MemoryScope {
    /// Memory scoped to the user across all projects.
    User,
    /// Memory scoped to a specific project.
    Project,
    /// Memory scoped to the local session only.
    Local,
}

impl std::fmt::Display for MemoryScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Project => write!(f, "project"),
            Self::Local => write!(f, "local"),
        }
    }
}

/// Extended thinking configuration for Claude.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    /// Maximum number of tokens to use for thinking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Effort level for thinking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<EffortLevel>,
}

/// Budget constraints for a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskBudget {
    /// Maximum number of conversation turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    /// Maximum number of tokens to consume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// Definition of a custom agent with its capabilities and configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentDefinition {
    /// Human-readable description of the agent.
    pub description: String,
    /// System prompt for the agent.
    pub prompt: String,
    /// Tools available to this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Model to use for this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Tools explicitly disallowed for this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disallowed_tools: Option<Vec<String>>,
    /// Skills available to this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    /// Memory scope for this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryScope>,
    /// MCP servers available to this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<serde_json::Value>>,
    /// Initial prompt to send when starting this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    /// Maximum number of turns for this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeAgentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsConfig>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<SystemPromptConfig>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, serde_json::Value>, // Simplified for now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default)]
    pub continue_conversation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_budget_usd: Option<f64>,
    #[serde(default)]
    pub disallowed_tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_model: Option<String>,
    #[serde(default)]
    pub betas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_prompt_tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
    #[serde(default)]
    pub add_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub extra_args: HashMap<String, Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_size: Option<usize>,
    #[serde(default)]
    pub include_partial_messages: bool,
    #[serde(default)]
    pub fork_session: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<HashMap<String, AgentDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setting_sources: Option<Vec<SettingSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxSettings>,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_thinking_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<serde_json::Value>,
    #[serde(default)]
    pub enable_file_checkpointing: bool,
    /// Effort level for Claude's responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<EffortLevel>,
    /// Extended thinking configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    /// Budget constraints for the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_budget: Option<TaskBudget>,
    /// Session identifier for tracking and resuming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Whether to use strict MCP configuration (no defaults).
    #[serde(default)]
    pub strict_mcp_config: bool,
    // Note: can_use_tool and hooks are handled differently in Rust (callbacks)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum PluginConfig {
    #[serde(rename = "local")]
    Local { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SandboxSettings {
    pub enabled: bool,
    pub auto_allow_bash_if_sandboxed: bool,
    pub excluded_commands: Vec<String>,
    pub allow_unsandboxed_commands: bool,
    pub network: Option<SandboxNetworkConfig>,
    pub ignore_violations: Option<SandboxIgnoreViolations>,
    pub enable_weaker_nested_sandbox: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SandboxNetworkConfig {
    pub allow_unix_sockets: Vec<String>,
    pub allow_all_unix_sockets: bool,
    pub allow_local_binding: bool,
    pub http_proxy_port: Option<u16>,
    pub socks_proxy_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SandboxIgnoreViolations {
    pub file: Vec<String>,
    pub network: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ToolsConfig {
    List(Vec<String>),
    Preset(ToolsPreset),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum SystemPromptConfig {
    Text(String),
    Preset(SystemPromptPreset),
}
