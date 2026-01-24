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
//! use claude_agent_transport::{Transport, SubprocessTransport};
//! use claude_agent_types::ClaudeAgentOptions;
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

use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions};

use crate::Transport;

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
/// use claude_agent_transport::{Transport, SubprocessTransport};
/// use claude_agent_types::ClaudeAgentOptions;
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
            use claude_agent_types::config::{SystemPromptConfig, SystemPromptPreset};
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

        // Tools
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

        // Limits
        if let Some(turns) = self.options.max_turns {
            cmd.arg("--max-turns");
            cmd.arg(turns.to_string());
        }
        if let Some(tokens) = self.options.max_thinking_tokens {
            cmd.arg("--max-thinking-tokens");
            cmd.arg(tokens.to_string());
        }

        // Context
        for dir in &self.options.add_dirs {
            cmd.arg("--add-dir");
            cmd.arg(dir.to_string_lossy().to_string());
        }

        // Session
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

        // Settings
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
                use crate::reader::MessageReader;
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
    use claude_agent_types::config::{PermissionMode, SystemPromptConfig, SystemPromptPreset};
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
}
