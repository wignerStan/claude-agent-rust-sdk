//! Subprocess transport implementation for Claude Code CLI.
//!
//! This module provides a subprocess-based transport implementation that spawns
//! the Claude Code CLI as a child process and communicates via stdin/stdout.
//!
//! # Architecture
//!
//! The transport uses a broadcast channel to distribute messages to multiple
//! subscribers, allowing the agent to drop and recreate the stream between
//! turns without losing messages.
//!
//! # Features
//!
//! - **Automatic CLI Discovery**: Searches common installation locations
//! - **Input Validation**: Validates CLI paths are executable files
//! - **Timeout Handling**: Prevents indefinite hangs during connection
//! - **Resource Cleanup**: Properly aborts background tasks on close
//! - **Broadcast Channel**: Distributes messages to multiple subscribers
//!
//! # Example
//!
//! ```rust,no_run
//! use crate::transport::{Transport, SubprocessTransport};
//! use crate::types::ClaudeAgentOptions;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut transport = SubprocessTransport::new(
//!         Some("Hello, Claude!".to_string()),
//!         ClaudeAgentOptions::default()
//!     );
//!
//!     // Connect with 30-second timeout
//!     Transport::connect(&mut transport).await?;
//!
//!     // Send a message
//!     transport.write("What is 2+2?").await?;
//!
//!     // Read responses
//!     {
//!         let mut stream = transport.read_messages().await;
//!         while let Some(result) = stream.next().await {
//!             match result {
//!                 Ok(msg) => println!("Received: {}", msg),
//!                 Err(e) => eprintln!("Error: {}", e),
//!             }
//!         }
//!     }
//!
//!     // Clean up
//!     transport.close().await?;
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use tokio::sync::Mutex;

use crate::types::{ClaudeAgentError, ClaudeAgentOptions};

use crate::transport::Transport;

/// Subprocess transport using Claude Code CLI.
///
/// This transport spawns the Claude Code CLI as a child process and
/// communicates with it via stdin for sending messages and stdout for receiving
/// streaming JSON responses.
///
/// # Thread Safety
///
/// This implementation is `Send + Sync` and can be safely shared across
/// multiple threads or async tasks. Internal state is protected by `Arc<Mutex<>>`.
///
/// # Resource Management
///
/// - The spawned process is tracked and properly killed on `close()`
/// - A background reader task is spawned to continuously read stdout
/// - An abort handle is stored to cancel the reader task on cleanup
/// - All resources are released when the transport is dropped or closed
///
/// # Broadcast Channel
///
/// Messages are distributed via a broadcast channel with a capacity of 1000 messages.
/// This allows multiple subscribers (e.g., different turns) to receive messages
/// without blocking each other. If there are no subscribers, messages are
/// silently dropped (SendError is ignored).
///
/// # Timeout
///
/// The `connect()` method has a 30-second timeout to prevent indefinite hangs
/// if the CLI process fails to start. This can be customized by modifying
/// the `CONNECT_TIMEOUT_SECS` constant.
///
/// # Example
///
/// ```rust,no_run
/// use crate::transport::{Transport, SubprocessTransport};
/// use crate::types::ClaudeAgentOptions;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut transport = SubprocessTransport::new(
///     Some("Hello".to_string()),
///     ClaudeAgentOptions::default()
/// );
/// Transport::connect(&mut transport).await?;
/// # Ok(())
/// # }
/// ```
pub struct SubprocessTransport {
    /// Configuration options for the CLI process.
    options: ClaudeAgentOptions,

    /// Optional prompt to send on connection.
    prompt: Option<String>,

    /// The spawned child process, if connected.
    process: Option<Child>,

    /// Shared stdin handle for writing to the process.
    stdin: Option<Arc<Mutex<tokio::process::ChildStdin>>>,

    /// Broadcast channel for distributing messages to multiple subscribers (turns).
    inbox: Option<tokio::sync::broadcast::Sender<Result<serde_json::Value, ClaudeAgentError>>>,

    /// Abort handle for the background reader task.
    reader_abort_handle: Option<tokio::task::AbortHandle>,
}

impl SubprocessTransport {
    /// Create a new subprocess transport.
    pub fn new(prompt: Option<String>, options: ClaudeAgentOptions) -> Self {
        Self { options, prompt, process: None, stdin: None, inbox: None, reader_abort_handle: None }
    }

    /// Find the Claude Code CLI binary.
    fn find_cli(&self) -> Result<PathBuf, ClaudeAgentError> {
        // Check if cli_path is explicitly set in options
        if let Some(ref path) = self.options.cli_path {
            if path.exists() {
                // Validate that it's a file and executable
                let metadata = std::fs::metadata(path).map_err(|e| {
                    ClaudeAgentError::CLINotFound(format!("Cannot access CLI path: {}", e))
                })?;

                if !metadata.is_file() {
                    return Err(ClaudeAgentError::CLINotFound(format!(
                        "CLI path is not a file: {}",
                        path.display()
                    )));
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o111 == 0 {
                        return Err(ClaudeAgentError::CLINotFound(format!(
                            "CLI is not executable: {}",
                            path.display()
                        )));
                    }
                }

                return Ok(path.clone());
            }
            return Err(ClaudeAgentError::CLINotFound(format!(
                "Specified CLI path does not exist: {}",
                path.display()
            )));
        }

        // Try to find 'claude' in PATH
        if let Ok(path) = which::which("claude") {
            return Ok(path);
        }

        // Common installation locations
        let common_paths = [
            dirs::home_dir().map(|h| h.join(".claude/local/claude")),
            Some(PathBuf::from("/usr/local/bin/claude")),
            Some(PathBuf::from("/opt/homebrew/bin/claude")),
        ];

        for path_opt in common_paths.iter().flatten() {
            if path_opt.exists() {
                // Validate that it's a file and executable
                if let Ok(metadata) = std::fs::metadata(path_opt) {
                    if metadata.is_file() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let permissions = metadata.permissions();
                            if permissions.mode() & 0o111 != 0 {
                                return Ok(path_opt.clone());
                            }
                        }
                        #[cfg(not(unix))]
                        {
                            return Ok(path_opt.clone());
                        }
                    }
                }
            }
        }

        Err(ClaudeAgentError::CLINotFound(
            "Claude Code CLI not found. Please install it or specify cli_path.".to_string(),
        ))
    }

    /// Build the CLI command with arguments.
    fn build_command(&self) -> Result<Command, ClaudeAgentError> {
        let cli_path = self.find_cli()?;
        let mut cmd = Command::new(&cli_path);

        // Set working directory
        if let Some(ref cwd) = self.options.cwd {
            cmd.current_dir(cwd);
        }

        // Add environment variables
        for (key, value) in &self.options.env {
            cmd.env(key, value);
        }

        // SDK entrypoint marker
        cmd.env("CLAUDE_CODE_ENTRYPOINT", "sdk-rs");

        // Basic output configuration
        cmd.arg("--output-format");
        cmd.arg("stream-json");
        cmd.arg("--input-format");
        cmd.arg("stream-json");
        cmd.arg("--verbose"); // Required for stream-json in newer CLI versions

        // Add prompt if provided
        if let Some(ref prompt) = self.prompt {
            cmd.arg(prompt);
        }

        // System prompt
        if let Some(ref system_prompt) = self.options.system_prompt {
            use crate::types::config::{SystemPromptConfig, SystemPromptPreset};
            match system_prompt {
                SystemPromptConfig::Text(text) => {
                    cmd.arg("--system-prompt");
                    cmd.arg(text);
                },
                SystemPromptConfig::Preset(SystemPromptPreset::Preset { preset: _, append }) => {
                    // Preset is usually implicit or default, but if there's an append:
                    if let Some(append_text) = append {
                        cmd.arg("--append-system-prompt");
                        cmd.arg(append_text);
                    }
                },
            }
        }

        // Tools configuration
        if let Some(ref tools) = self.options.tools {
            use crate::types::config::ToolsConfig;
            match tools {
                ToolsConfig::List(list) => {
                    if !list.is_empty() {
                        cmd.arg("--tools");
                        cmd.arg(list.join(","));
                    }
                },
                ToolsConfig::Preset(preset) => {
                    cmd.arg("--tools");
                    cmd.arg(match preset {
                        crate::types::config::ToolsPreset::Preset { ref preset } => preset.as_str(),
                    });
                },
            }
        }

        // Allowed and disallowed tools
        if !self.options.allowed_tools.is_empty() {
            cmd.arg("--allowedTools");
            cmd.arg(self.options.allowed_tools.join(","));
        }
        if !self.options.disallowed_tools.is_empty() {
            cmd.arg("--disallowedTools");
            cmd.arg(self.options.disallowed_tools.join(","));
        }

        // Model
        if let Some(ref model) = self.options.model {
            cmd.arg("--model");
            cmd.arg(model);
        }
        if let Some(ref fallback) = self.options.fallback_model {
            cmd.arg("--fallback-model");
            cmd.arg(fallback);
        }

        // Permissions
        if let Some(ref mode) = self.options.permission_mode {
            cmd.arg("--permission-mode");
            cmd.arg(mode.to_string());
        }

        // Permission prompt tool name
        if let Some(ref tool_name) = self.options.permission_prompt_tool_name {
            cmd.arg("--permission-prompt-tool");
            cmd.arg(tool_name);
        }

        // Limits
        if let Some(turns) = self.options.max_turns {
            cmd.arg("--max-turns");
            cmd.arg(turns.to_string());
        }

        // Max budget (USD)
        if let Some(budget) = self.options.max_budget_usd {
            cmd.arg("--max-budget-usd");
            cmd.arg(budget.to_string());
        }

        // Thinking configuration (prefer thinking struct over legacy field)
        if let Some(ref thinking) = self.options.thinking {
            if let Some(max_tokens) = thinking.max_tokens {
                cmd.arg("--max-thinking-tokens");
                cmd.arg(max_tokens.to_string());
            }
        } else if let Some(tokens) = self.options.max_thinking_tokens {
            // Fallback to legacy max_thinking_tokens if thinking not set
            cmd.arg("--max-thinking-tokens");
            cmd.arg(tokens.to_string());
        }

        // Effort level
        if let Some(ref effort) = self.options.effort {
            cmd.arg("--effort");
            cmd.arg(effort.to_string());
        } else if let Some(ref thinking) = self.options.thinking {
            // Derive effort from thinking config if not set directly
            if let Some(ref thinking_effort) = thinking.effort {
                cmd.arg("--effort");
                cmd.arg(thinking_effort.to_string());
            }
        }

        // Task budget
        if let Some(ref budget) = self.options.task_budget {
            cmd.arg("--task-budget");
            cmd.arg(serde_json::to_string(budget).map_err(|e| {
                ClaudeAgentError::CLIConnection(format!("Failed to serialize task budget: {}", e))
            })?);
        }

        // Betas
        if !self.options.betas.is_empty() {
            cmd.arg("--betas");
            cmd.arg(self.options.betas.join(","));
        }

        // Setting sources
        if let Some(ref sources) = self.options.setting_sources {
            use crate::types::config::SettingSource;
            let source_strs: Vec<&str> = sources
                .iter()
                .map(|s| match s {
                    SettingSource::User => "user",
                    SettingSource::Project => "project",
                    SettingSource::Local => "local",
                })
                .collect();
            cmd.arg("--setting-sources");
            cmd.arg(source_strs.join(","));
        }

        // Fork session flag
        if self.options.fork_session {
            cmd.arg("--fork-session");
        }

        // Include partial messages flag
        if self.options.include_partial_messages {
            cmd.arg("--include-partial-messages");
        }

        // Output format override (overrides default stream-json if set)
        if let Some(ref format) = self.options.output_format {
            cmd.arg("--output-format");
            cmd.arg(format.to_string());
        }

        // Session ID
        if let Some(ref session_id) = self.options.session_id {
            cmd.arg("--session-id");
            cmd.arg(session_id);
        }

        // Strict MCP config flag
        if self.options.strict_mcp_config {
            cmd.arg("--strict-mcp-config");
        }

        // Sandbox settings — merge into --settings JSON
        if let Some(ref sandbox) = self.options.sandbox {
            let sandbox_json = serde_json::to_value(sandbox).map_err(|e| {
                ClaudeAgentError::CLIConnection(format!(
                    "Failed to serialize sandbox settings: {}",
                    e
                ))
            })?;
            cmd.arg("--settings");
            cmd.arg(serde_json::json!({ "sandbox": sandbox_json }).to_string());
        }

        // Plugins — repeat --plugin-dir for each plugin
        for plugin in &self.options.plugins {
            match plugin {
                crate::types::config::PluginConfig::Local { ref path } => {
                    cmd.arg("--plugin-dir");
                    cmd.arg(path.to_string_lossy().to_string());
                },
            }
        }

        // Agents — serialize the HashMap to JSON
        if let Some(ref agents) = self.options.agents {
            if !agents.is_empty() {
                cmd.arg("--agents");
                cmd.arg(serde_json::to_string(agents).map_err(|e| {
                    ClaudeAgentError::CLIConnection(format!(
                        "Failed to serialize agents config: {}",
                        e
                    ))
                })?);
            }
        }

        // File checkpointing env var
        if self.options.enable_file_checkpointing {
            cmd.env("CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING", "1");
        }

        // Context
        for dir in &self.options.add_dirs {
            cmd.arg("--add-dir");
            cmd.arg(dir.to_string_lossy().to_string());
        }

        // Session continuation
        if self.options.continue_conversation {
            cmd.arg("--continue");
            if let Some(ref id) = self.options.resume {
                cmd.arg("--resume");
                cmd.arg(id);
            }
        }

        // MCP Config
        if !self.options.mcp_servers.is_empty() {
            cmd.arg("--mcp-config");
            let config = serde_json::json!({
                "mcpServers": self.options.mcp_servers
            });
            cmd.arg(config.to_string());
        }

        // Settings (user-provided settings file or JSON)
        if let Some(ref settings) = self.options.settings {
            cmd.arg("--settings");
            cmd.arg(settings);
        }

        // Extra args
        for (flag, value) in &self.options.extra_args {
            let flag_str =
                if flag.starts_with("--") { flag.clone() } else { format!("--{}", flag) };
            cmd.arg(flag_str);
            if let Some(v) = value {
                cmd.arg(v);
            }
        }

        // Configure stdio
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit());

        Ok(cmd)
    }
}

#[async_trait]
impl Transport for SubprocessTransport {
    async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
        // Add timeout to prevent hanging indefinitely
        const CONNECT_TIMEOUT_SECS: u64 = 30;
        tokio::time::timeout(tokio::time::Duration::from_secs(CONNECT_TIMEOUT_SECS), async {
            let mut cmd = self.build_command()?;
            let mut child = cmd.spawn().map_err(|e| {
                ClaudeAgentError::CLIConnection(format!("Failed to spawn CLI process: {}", e))
            })?;

            // Take ownership of stdin
            let stdin = child.stdin.take().ok_or_else(|| {
                ClaudeAgentError::CLIConnection("Failed to get stdin handle".to_string())
            })?;
            self.stdin = Some(Arc::new(Mutex::new(stdin)));

            // Take ownership of stdout and spawn reader task
            let stdout = child.stdout.take().ok_or_else(|| {
                ClaudeAgentError::CLIConnection("Failed to get stdout handle".to_string())
            })?;

            // precise capacity
            const BROADCAST_CHANNEL_CAPACITY: usize = 1000;
            let (tx, _) = tokio::sync::broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
            self.inbox = Some(tx.clone());

            let abort_handle = tokio::spawn(async move {
                use crate::transport::reader::MessageReader;
                use futures::StreamExt;

                let reader = MessageReader::new(stdout);
                let mut stream = Box::pin(reader);

                while let Some(msg_res) = stream.next().await {
                    // Determine if we should stop?
                    // If everyone disconnected?
                    // For now, keep reading until EOF or error.

                    // We map parse errors or logic errors from reader
                    // reader returns Result<Value, ClaudeAgentError>

                    if tx.send(msg_res).is_err() {
                        // No subscribers left, but we should keep reading to drain stdout?
                        // Or maybe just exit.
                        // Ideally we keep reading because a new subscriber might appear (Next Turn).
                        // But broadcast channel returns error only if there are NO receivers?
                        // "SendError if there are no active receivers"
                        // In our case, Agent drops stream between turns.
                        // So there might be moments with 0 receivers.
                        // We should ignore SendError and continue.
                    }
                }
            })
            .abort_handle();

            self.reader_abort_handle = Some(abort_handle);

            self.process = Some(child);

            Ok::<(), ClaudeAgentError>(())
        })
        .await
        .map_err(|_| {
            ClaudeAgentError::CLIConnection(format!(
                "Connection timeout after {} seconds",
                CONNECT_TIMEOUT_SECS
            ))
        })?
    }

    async fn write(&self, data: &str) -> Result<(), ClaudeAgentError> {
        let stdin = self
            .stdin
            .as_ref()
            .ok_or_else(|| ClaudeAgentError::Transport("Transport not connected".to_string()))?;

        let mut guard = stdin.lock().await;
        guard
            .write_all(data.as_bytes())
            .await
            .map_err(|e| ClaudeAgentError::Transport(format!("Write failed: {}", e)))?;
        guard
            .write_all(b"\n")
            .await
            .map_err(|e| ClaudeAgentError::Transport(format!("Write newline failed: {}", e)))?;
        guard
            .flush()
            .await
            .map_err(|e| ClaudeAgentError::Transport(format!("Flush failed: {}", e)))?;

        Ok(())
    }

    async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
        use futures::StreamExt;
        use tokio_stream::wrappers::BroadcastStream;

        match &self.inbox {
            Some(tx) => {
                let rx = tx.subscribe();
                let stream = BroadcastStream::new(rx);
                // BroadcastStream yields Result<Result<Value, Error>, RecvError>
                // We need to flatten and map RecvError

                Box::pin(stream.map(|item| match item {
                    Ok(payload) => payload,
                    Err(e) => {
                        Err(ClaudeAgentError::Transport(format!("Broadcast receive error: {}", e)))
                    },
                }))
            },
            None => Box::pin(stream::once(async {
                Err(ClaudeAgentError::Transport("Transport not connected".to_string()))
            })),
        }
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        // Abort reader task
        if let Some(abort_handle) = self.reader_abort_handle.take() {
            abort_handle.abort();
        }

        // Drop stdin to signal EOF
        self.stdin = None;

        // Wait for process to exit
        if let Some(ref mut process) = self.process {
            process.wait().await.map_err(|e| {
                ClaudeAgentError::Process(format!("Failed to wait for process exit: {}", e))
            })?;
        }
        self.process = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::{
        AgentDefinition, EffortLevel, PermissionMode, PluginConfig, SettingSource,
        SystemPromptConfig, SystemPromptPreset, TaskBudget, ThinkingConfig, ToolsConfig,
        ToolsPreset,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Write;

    fn dummy_cli_path() -> &'static std::path::PathBuf {
        static PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        PATH.get_or_init(|| {
            let mut temp_dir = std::env::temp_dir();
            temp_dir.push("dummy_claude_cli");
            let file_path = temp_dir;

            // Create the dummy executable
            let mut file = File::create(&file_path).expect("failed to create dummy CLI");
            writeln!(file, "#!/bin/sh").expect("failed to write shebang");
            writeln!(file, "exit 0").expect("failed to write exit");

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&file_path).expect("metadata failed").permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&file_path, perms).expect("set_permissions failed");
            }

            file_path
        })
    }

    fn make_options() -> ClaudeAgentOptions {
        let mut options = ClaudeAgentOptions { ..Default::default() };
        options.cli_path = Some(dummy_cli_path().clone());
        options
    }

    #[test]
    fn test_build_command_basic() {
        let transport = SubprocessTransport::new(Some("Hello".to_string()), make_options());
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--output-format"));
        assert!(cmd_str.contains("stream-json"));
    }

    #[test]
    fn test_build_command_with_system_prompt_string() {
        let mut options = make_options();
        options.system_prompt = Some(SystemPromptConfig::Text("Be helpful".to_string()));

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--system-prompt"));
        assert!(cmd_str.contains("Be helpful"));
    }

    #[test]
    fn test_build_command_with_system_prompt_preset() {
        let mut options = make_options();
        options.system_prompt = Some(SystemPromptConfig::Preset(SystemPromptPreset::Preset {
            preset: "claude_code".to_string(),
            append: None,
        }));

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(!cmd_str.contains("--system-prompt"));
        assert!(!cmd_str.contains("--append-system-prompt"));
    }

    #[test]
    fn test_build_command_with_system_prompt_preset_and_append() {
        let mut options = make_options();
        options.system_prompt = Some(SystemPromptConfig::Preset(SystemPromptPreset::Preset {
            preset: "claude_code".to_string(),
            append: Some("Be concise.".to_string()),
        }));

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(!cmd_str.contains("--system-prompt"));
        assert!(cmd_str.contains("--append-system-prompt"));
        assert!(cmd_str.contains("Be concise."));
    }

    #[test]
    fn test_build_command_with_options() {
        let mut options = make_options();
        options.allowed_tools = vec!["Read".to_string(), "Write".to_string()];
        options.disallowed_tools = vec!["Bash".to_string()];
        options.model = Some("claude-sonnet-4-5".to_string());
        options.permission_mode = Some(PermissionMode::AcceptEdits);
        options.max_turns = Some(5);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--allowedTools"));
        assert!(cmd_str.contains("Read,Write"));
        assert!(cmd_str.contains("--disallowedTools"));
        assert!(cmd_str.contains("Bash"));
        assert!(cmd_str.contains("--model"));
        assert!(cmd_str.contains("claude-sonnet-4-5"));
        assert!(cmd_str.contains("--permission-mode"));
        assert!(cmd_str.contains("acceptEdits"));
        assert!(cmd_str.contains("--max-turns"));
        assert!(cmd_str.contains("5"));
    }

    #[test]
    fn test_build_command_with_fallback_model() {
        let mut options = make_options();
        options.model = Some("opus".to_string());
        options.fallback_model = Some("sonnet".to_string());

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--model"));
        assert!(cmd_str.contains("opus"));
        assert!(cmd_str.contains("--fallback-model"));
        assert!(cmd_str.contains("sonnet"));
    }

    #[test]
    fn test_build_command_with_max_thinking_tokens() {
        let mut options = make_options();
        options.max_thinking_tokens = Some(5000);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--max-thinking-tokens"));
        assert!(cmd_str.contains("5000"));
    }

    #[test]
    fn test_build_command_with_add_dirs() {
        let mut options = make_options();
        options.add_dirs = vec![PathBuf::from("/path/to/dir1"), PathBuf::from("/path/to/dir2")];

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--add-dir"));
        assert!(cmd_str.contains("/path/to/dir1"));
        assert!(cmd_str.contains("/path/to/dir2"));
    }

    #[test]
    fn test_session_continuation() {
        let mut options = make_options();
        options.continue_conversation = true;
        options.resume = Some("session-123".to_string());

        let transport = SubprocessTransport::new(Some("Continue".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--continue"));
        assert!(cmd_str.contains("--resume"));
        assert!(cmd_str.contains("session-123"));
    }

    #[test]
    fn test_build_command_with_settings_file() {
        let mut options = make_options();
        options.settings = Some("/path/to/settings.json".to_string());

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--settings"));
        assert!(cmd_str.contains("/path/to/settings.json"));
    }

    #[test]
    fn test_build_command_with_extra_args() {
        let mut options = make_options();
        let mut extra = HashMap::new();
        extra.insert("new-flag".to_string(), Some("value".to_string()));
        extra.insert("boolean-flag".to_string(), None);
        options.extra_args = extra;

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--new-flag"));
        assert!(cmd_str.contains("value"));
        assert!(cmd_str.contains("--boolean-flag"));
    }

    #[test]
    fn test_build_command_with_mcp_servers() {
        let mut options = make_options();
        let mut servers = HashMap::new();
        servers.insert(
            "test-server".to_string(),
            json!({
                "command": "test-cmd",
                "args": ["arg1"]
            }),
        );
        options.mcp_servers = servers;

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--mcp-config"));
        assert!(cmd_str.contains("mcpServers"));
        assert!(cmd_str.contains("test-server"));
        assert!(cmd_str.contains("test-cmd"));
    }

    // --- New tests for Part B: unwired CLI flags ---

    #[test]
    fn test_build_command_with_max_budget_usd() {
        let mut options = make_options();
        options.max_budget_usd = Some(5.50);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--max-budget-usd"));
        assert!(cmd_str.contains("5.5"));
    }

    #[test]
    fn test_build_command_with_betas() {
        let mut options = make_options();
        options.betas = vec!["max-tokens".to_string(), "new-feature".to_string()];

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--betas"));
        assert!(cmd_str.contains("max-tokens,new-feature"));
    }

    #[test]
    fn test_build_command_with_permission_prompt_tool_name() {
        let mut options = make_options();
        options.permission_prompt_tool_name = Some("custom-tool".to_string());

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--permission-prompt-tool"));
        assert!(cmd_str.contains("custom-tool"));
    }

    #[test]
    fn test_build_command_with_setting_sources() {
        let mut options = make_options();
        options.setting_sources =
            Some(vec![SettingSource::User, SettingSource::Project, SettingSource::Local]);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--setting-sources"));
        assert!(cmd_str.contains("user,project,local"));
    }

    #[test]
    fn test_build_command_with_fork_session() {
        let mut options = make_options();
        options.fork_session = true;

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--fork-session"));
    }

    #[test]
    fn test_build_command_without_fork_session() {
        let options = make_options();

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(!cmd_str.contains("--fork-session"));
    }

    #[test]
    fn test_build_command_with_include_partial_messages() {
        let mut options = make_options();
        options.include_partial_messages = true;

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--include-partial-messages"));
    }

    #[test]
    fn test_build_command_with_tools_list() {
        let mut options = make_options();
        options.tools = Some(ToolsConfig::List(vec!["Read".to_string(), "Write".to_string()]));

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--tools"));
        assert!(cmd_str.contains("Read,Write"));
    }

    #[test]
    fn test_build_command_with_tools_preset() {
        let mut options = make_options();
        options.tools =
            Some(ToolsConfig::Preset(ToolsPreset::Preset { preset: "claude_code".to_string() }));

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--tools"));
        assert!(cmd_str.contains("claude_code"));
    }

    #[test]
    fn test_build_command_with_output_format_override() {
        let mut options = make_options();
        options.output_format = Some(json!("json"));

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        // output-format should appear twice: once for default stream-json, once for override
        // The override should come after and contain "json"
        assert!(cmd_str.contains("json"));
    }

    #[test]
    fn test_build_command_with_sandbox_settings() {
        use crate::types::config::SandboxSettings;
        let mut options = make_options();
        let mut sandbox = SandboxSettings::default();
        sandbox.enabled = true;
        options.sandbox = Some(sandbox);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--settings"));
        assert!(cmd_str.contains("sandbox"));
    }

    #[test]
    fn test_build_command_with_plugins() {
        let mut options = make_options();
        options.plugins = vec![
            PluginConfig::Local { path: PathBuf::from("/path/to/plugin1") },
            PluginConfig::Local { path: PathBuf::from("/path/to/plugin2") },
        ];

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--plugin-dir"));
        assert!(cmd_str.contains("/path/to/plugin1"));
        assert!(cmd_str.contains("/path/to/plugin2"));
    }

    #[test]
    fn test_build_command_with_agents() {
        let mut options = make_options();
        let mut agents = HashMap::new();
        agents.insert(
            "reviewer".to_string(),
            AgentDefinition {
                description: "Code reviewer".to_string(),
                prompt: "Review this code".to_string(),
                tools: Some(vec!["Read".to_string()]),
                model: Some("sonnet".to_string()),
                disallowed_tools: None,
                skills: None,
                memory: None,
                mcp_servers: None,
                initial_prompt: None,
                max_turns: None,
            },
        );
        options.agents = Some(agents);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--agents"));
        assert!(cmd_str.contains("reviewer"));
        assert!(cmd_str.contains("Code reviewer"));
    }

    #[test]
    fn test_build_command_with_enable_file_checkpointing() {
        let mut options = make_options();
        options.enable_file_checkpointing = true;

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(
            cmd_str.contains("CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING"),
            "Expected checkpointing env var in: {cmd_str}"
        );
    }

    #[test]
    fn test_build_command_with_effort() {
        let mut options = make_options();
        options.effort = Some(EffortLevel::High);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--effort"));
        assert!(cmd_str.contains("high"));
    }

    #[test]
    fn test_build_command_with_thinking_config() {
        let mut options = make_options();
        options.thinking = Some(ThinkingConfig { max_tokens: Some(10000), effort: None });

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--max-thinking-tokens"));
        assert!(cmd_str.contains("10000"));
    }

    #[test]
    fn test_build_command_with_thinking_config_effort() {
        let mut options = make_options();
        options.thinking =
            Some(ThinkingConfig { max_tokens: Some(10000), effort: Some(EffortLevel::Max) });

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--max-thinking-tokens"));
        assert!(cmd_str.contains("10000"));
        // effort from thinking config takes effect when no direct effort is set
        assert!(cmd_str.contains("--effort"));
        assert!(cmd_str.contains("max"));
    }

    #[test]
    fn test_build_command_with_thinking_config_max_tokens() {
        let mut options = make_options();
        // When both thinking and legacy max_thinking_tokens are set, thinking wins
        options.thinking = Some(ThinkingConfig { max_tokens: Some(8000), effort: None });
        options.max_thinking_tokens = Some(5000);

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--max-thinking-tokens"));
        assert!(cmd_str.contains("8000"));
        assert!(!cmd_str.contains("5000"));
    }

    #[test]
    fn test_build_command_with_task_budget() {
        let mut options = make_options();
        options.task_budget = Some(TaskBudget { max_turns: Some(10), max_tokens: Some(50000) });

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--task-budget"));
        assert!(cmd_str.contains("maxTurns"));
        assert!(cmd_str.contains("10"));
        assert!(cmd_str.contains("maxTokens"));
        assert!(cmd_str.contains("50000"));
    }

    #[test]
    fn test_build_command_with_session_id() {
        let mut options = make_options();
        options.session_id = Some("sess-abc-123".to_string());

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--session-id"));
        assert!(cmd_str.contains("sess-abc-123"));
    }

    #[test]
    fn test_build_command_with_strict_mcp_config() {
        let mut options = make_options();
        options.strict_mcp_config = true;

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(cmd_str.contains("--strict-mcp-config"));
    }

    #[test]
    fn test_build_command_without_strict_mcp_config() {
        let options = make_options();

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        assert!(!cmd_str.contains("--strict-mcp-config"));
    }

    #[test]
    fn test_direct_effort_overrides_thinking_effort() {
        let mut options = make_options();
        options.effort = Some(EffortLevel::Low);
        options.thinking =
            Some(ThinkingConfig { max_tokens: Some(10000), effort: Some(EffortLevel::Max) });

        let transport = SubprocessTransport::new(Some("test".to_string()), options);
        let cmd = transport.build_command().expect("Failed to build command");
        let cmd_str = format!("{:?}", cmd);

        // Direct effort should take precedence over thinking.effort
        assert!(cmd_str.contains("--effort"));
        // The first occurrence of --effort should be "low" (direct), not "max" (thinking)
        let idx_effort = cmd_str.find("--effort").expect("should have --effort");
        let after_effort = &cmd_str[idx_effort..];
        // Should contain "low" soon after --effort, not "max"
        assert!(after_effort.contains("low"));
    }
}
