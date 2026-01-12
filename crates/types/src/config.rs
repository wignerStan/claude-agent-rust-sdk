use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
// Hook types are handled via callbacks in Rust

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    Default,
    AcceptEdits,
    Plan,
    BypassPermissions,
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::AcceptEdits => write!(f, "acceptEdits"),
            Self::Plan => write!(f, "plan"),
            Self::BypassPermissions => write!(f, "bypassPermissions"),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentDefinition {
    pub description: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
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
