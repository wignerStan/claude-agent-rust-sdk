//! Core agent implementation for Claude Agent SDK.
//!
//! This module provides the main `ClaudeAgent` struct that orchestrates
//! communication with Claude Code CLI, manages sessions, handles tool calls,
//! and integrates with MCP (Model Context Protocol) servers.
//!
//! # Architecture
//!
//! The agent is the primary entry point for interacting with Claude. It manages:
//!
//! - **Transport Layer**: Handles subprocess spawning and message streaming
//! - **Session Management**: Tracks conversation state across multiple turns
//! - **MCP Integration**: Manages Model Context Protocol servers for tool calls
//! - **Control Protocol**: Bidirectional communication with Claude Code CLI
//! - **Hook System**: Extensible callbacks for tool execution and other events
//! - **Permission System**: Handles user approvals for file operations
//!
//! # Lifecycle
//!
//! 1. **Initialization**: Create agent with options
//! 2. **Connection**: Connect to Claude Code CLI via transport
//! 3. **Query**: Send prompts and receive streaming responses
//! 4. **Session Management**: Create/retrieve/update sessions
//! 5. **Cleanup**: Close transport and release resources
//!
//! # Example
//!
//! ```rust,no_run
//! use claude_agent_core::{ClaudeAgent, ClaudeAgentOptions};
//! use claude_agent_types::Message;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let options = ClaudeAgentOptions::default();
//!     let mut agent = ClaudeAgent::new(options);
//!
//!     // Connect to Claude
//!     agent.connect(None).await?;
//!
//!     // Send a query and process responses
//!     {
//!         let mut stream = agent.query("What is 2+2?").await?;
//!         while let Some(result) = stream.next().await {
//!             match result {
//!                 Ok(msg) => println!("Response: {:?}", msg),
//!                 Err(e) => eprintln!("Error: {}", e),
//!             }
//!         }
//!     }
//!
//!     // Cleanup
//!     agent.disconnect().await?;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;

use futures::stream::BoxStream;
use futures::StreamExt;

use claude_agent_mcp::McpServerManager;
use claude_agent_transport::{SubprocessTransport, Transport};
use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions, Message};

use crate::control::{ControlProtocol, ControlResponse};
use crate::hooks::HookRegistry;
use crate::permissions::PermissionHandler;
use crate::session::{Session, SessionManager};

/// The core Claude Agent.
///
/// This struct orchestrates all components of the Claude Agent SDK:
/// - Transport management (spawning and communicating with Claude Code CLI)
/// - Session management (tracking conversation state)
/// - MCP server integration (for tool calls)
/// - Control protocol (bidirectional communication with Claude)
/// - Hook system (extensible callbacks)
/// - Permission handling (user approvals for file operations)
///
/// # Thread Safety
///
/// The agent is `Send + Sync` and can be safely shared across
/// multiple threads or async tasks. Internal state is protected by `Arc<Mutex<>>`
/// for concurrent access.
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent_core::{ClaudeAgent, ClaudeAgentOptions};
/// use claude_agent_types::Message;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let options = ClaudeAgentOptions::default();
/// let mut agent = ClaudeAgent::new(options);
///
/// // Connect and query
/// agent.connect(None).await?;
/// let mut stream = agent.query("Hello, Claude!").await?;
///
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(msg) => println!("Response: {:?}", msg),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
pub struct ClaudeAgent {
    /// Configuration options for the agent.
    options: ClaudeAgentOptions,

    /// The transport layer for communicating with Claude Code CLI.
    ///
    /// This is set via `set_transport()` and used for all
    /// communication. If `None`, the agent will automatically spawn a
    /// `SubprocessTransport` on the first `connect()` call.
    transport: Option<Box<dyn Transport>>,

    /// Session manager for tracking conversation state.
    ///
    /// Manages multiple sessions with checkpoints and allows resuming
    /// conversations across different turns.
    session_manager: SessionManager,

    /// Registry for hook callbacks.
    ///
    /// Allows extending agent behavior with custom callbacks for
    /// tool execution, permission handling, and other events.
    hook_registry: HookRegistry,

    /// Handler for permission requests (file operations).
    ///
    /// Processes permission prompts from Claude and coordinates with
    /// the hook system for user approval.
    permission_handler: PermissionHandler,

    /// Manager for MCP (Model Context Protocol) servers.
    ///
    /// Manages registration and tool calls to MCP servers.
    /// Enables Claude to use external tools and context.
    mcp_manager: McpServerManager,

    /// Optional control protocol for bidirectional communication.
    ///
    /// When set, allows the agent to send control messages
    /// (e.g., for tool execution results) and receive responses.
    /// Uses a channel-based architecture for message passing.
    control_protocol: Option<Arc<ControlProtocol>>,

    /// Receiver for control protocol messages.
    ///
    /// Receives control requests and responses from the control protocol.
    /// Protected by `Arc<Mutex<>>` for thread-safe access.
    control_rx:
        Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::control::ControlRequest>>>,

    /// Optional initialization data sent on connection.
    ///
    /// This data is sent to Claude on connection to provide
    /// context for the conversation (e.g., previous messages, system state).
    initialization_data: Arc<tokio::sync::Mutex<Option<serde_json::Value>>>,
}

impl ClaudeAgent {
    /// Create a new Claude Agent.
    pub fn new(options: ClaudeAgentOptions) -> Self {
        let (protocol, rx) = ControlProtocol::new();
        Self {
            options,
            transport: None,
            session_manager: SessionManager::new(),
            hook_registry: HookRegistry::new(),
            permission_handler: PermissionHandler::new(),
            mcp_manager: McpServerManager::new(),
            control_protocol: Some(Arc::new(protocol)),
            control_rx: Arc::new(tokio::sync::Mutex::new(rx)),
            initialization_data: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Set the transport implementation.
    ///
    /// Useful for testing with mock transports or using custom transport implementations.
    pub fn set_transport(&mut self, transport: Box<dyn Transport>) {
        self.transport = Some(transport);
    }

    /// Connect to Claude Code CLI.
    pub async fn connect(&mut self, prompt: Option<&str>) -> Result<(), ClaudeAgentError> {
        if self.transport.is_none() {
            let transport =
                SubprocessTransport::new(prompt.map(|s| s.to_string()), self.options.clone());
            self.transport = Some(Box::new(transport));
        }

        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| ClaudeAgentError::Transport("Transport not initialized".to_string()))?;
        transport.connect().await?;

        // Create session
        self.session_manager.create_session();

        Ok(())
    }

    /// Execute a query and return a stream of messages.
    pub async fn query(
        &mut self,
        prompt: &str,
    ) -> Result<BoxStream<'_, Result<Message, ClaudeAgentError>>, ClaudeAgentError> {
        // Connect if not already connected
        if self.transport.is_none() {
            self.connect(None).await?;
        }

        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| ClaudeAgentError::Transport("Transport not connected".to_string()))?;

        // Write the prompt to the transport
        use serde_json::json;

        // Construct a proper UserMessage for the stream-json protocol
        // We manually construct it to ensure structure matches what CLI expects:
        // { "type": "user", "message": { "role": "user", "content": ... } }
        let user_msg = json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": prompt
                    }
                ]
            }
        });

        let msg_str = serde_json::to_string(&user_msg).unwrap_or_else(|_| prompt.to_string());

        transport.write(&msg_str).await?;

        let mut json_stream = transport.read_messages().await;

        // Create references to fields we need in the stream to avoid capturing &mut self
        let mcp_manager = &self.mcp_manager;
        let control_protocol = self.control_protocol.as_ref().cloned();
        let control_rx_mutex = self.control_rx.clone();
        let initialization_data_mutex = self.initialization_data.clone();

        // Use async-stream for the complex loop
        let stream = async_stream::stream! {
            loop {
                // We need to lock the receiver to wait for messages
                // This means we hold the lock while waiting for EITHER transport OR control
                // This is safe because no one else is contending for control_rx during query
                let mut control_guard = control_rx_mutex.lock().await;

                tokio::select! {
                     // Handle incoming messages from Transport
                     maybe_msg = json_stream.next() => {
                         match maybe_msg {
                             Some(result) => {
                                 let value = match result {
                                     Ok(v) => v,
                                     Err(e) => {
                                         yield Err(e);
                                         break;
                                     }
                                 };

                                 let msg_type = value.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");

                                 if msg_type == "control_request" {
                                      let req_id = value.get("request_id").and_then(|s| s.as_str()).unwrap_or("unknown");
                                      let req_payload = value.get("request").cloned().unwrap_or(serde_json::Value::Null);
                                      let subtype = req_payload.get("subtype").and_then(|s| s.as_str()).unwrap_or("unknown");

                                      let response_data = match subtype {
                                          "mcp_message" => {
                                              let server_name = req_payload.get("server_name").and_then(|s| s.as_str());
                                              let message = req_payload.get("message");

                                              if let (Some(name), Some(msg)) = (server_name, message) {
                                                  if let Some(server) = mcp_manager.get(name) {
                                                      match server.handle_client_message(msg.clone()).await {
                                                          Ok(res) => res,
                                                          Err(e) => serde_json::json!({"error": e.to_string()})
                                                      }
                                                  } else {
                                                       serde_json::json!({"error": format!("Server not found: {}", name)})
                                                  }
                                              } else {
                                                  serde_json::json!({"error": "Invalid mcp_message payload"})
                                              }
                                          },
                                          "initialize" | "set_permission_mode" | "set_model" | "rewind_files" => {
                                               serde_json::json!({"status": "not_implemented"})
                                          }
                                          _ => {
                                               serde_json::json!({"error": format!("Unsupported control subtype: {}", subtype)})
                                          }
                                      };

                                       let response = serde_json::json!({
                                          "type": "control_response",
                                          "response": {
                                              "subtype": "success",
                                              "request_id": req_id,
                                              "response": response_data
                                          }
                                      });

                                      let response_str = serde_json::to_string(&response).unwrap_or_default();
                                      if let Err(e) = transport.write(&response_str).await {
                                           yield Err(e);
                                           break;
                                      }
                                 } else if msg_type == "control_response" {
                                     if let Some(cp) = &control_protocol {
                                          let req_id = value.get("request_id").and_then(|s| s.as_str()).unwrap_or("");
                                          let resp = ControlResponse {
                                              request_id: req_id.to_string(),
                                              success: true,
                                              response: Some(value.clone()),
                                              error: None
                                          };
                                          let _ = cp.handle_response(resp).await;
                                     }
                                 } else {
                                     // Check if it's the init message
                                     if msg_type == "system" && value.get("subtype").and_then(|t| t.as_str()) == Some("init") {
                                         let mut init_guard = initialization_data_mutex.lock().await;
                                         *init_guard = value.get("data").cloned();
                                     }

                                     match serde_json::from_value(value) {
                                         Ok(msg) => yield Ok(msg),
                                         Err(e) => {
                                             yield Err(ClaudeAgentError::MessageParse(format!("Failed to parse message: {}", e)));
                                         }
                                     }
                                 }
                             },
                             None => break, // Stream anded
                         }
                     },

                     Some(req) = control_guard.recv() => {
                         let req_json = serde_json::json!({
                             "type": "control_request",
                             "request_id": req.request_id,
                             "request": match req.request {
                                 crate::control::ControlRequestType::Interrupt => serde_json::json!({"subtype": "interrupt"}),
                                 crate::control::ControlRequestType::SetPermissionMode { mode } => serde_json::json!({"subtype": "set_permission_mode", "mode": mode}),
                                 crate::control::ControlRequestType::SetModel { model } => serde_json::json!({"subtype": "set_model", "model": model}),
                                 crate::control::ControlRequestType::RewindFiles { user_message_id } => serde_json::json!({"subtype": "rewind_files", "user_message_id": user_message_id}),
                                 _ => serde_json::Value::Null
                             }
                         });

                         let req_str = serde_json::to_string(&req_json).unwrap_or_default();
                         if let Err(e) = transport.write(&req_str).await {
                             yield Err(e);
                             break;
                         }
                     }
                }
            }

        };

        Ok(Box::pin(stream))
    }

    /// Get information about the connected server.
    pub async fn get_server_info(&self) -> Option<serde_json::Value> {
        let guard = self.initialization_data.lock().await;
        guard.clone()
    }

    /// Send interrupt signal.
    pub async fn interrupt(&self) -> Result<ControlResponse, ClaudeAgentError> {
        let protocol = self.control_protocol.as_ref().ok_or_else(|| {
            ClaudeAgentError::ControlProtocol("Control protocol not initialized".to_string())
        })?;
        protocol.interrupt().await
    }

    /// Set permission mode.
    pub async fn set_permission_mode(
        &self,
        mode: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        let protocol = self.control_protocol.as_ref().ok_or_else(|| {
            ClaudeAgentError::ControlProtocol("Control protocol not initialized".to_string())
        })?;
        protocol.set_permission_mode(mode).await
    }

    /// Set model.
    pub async fn set_model(
        &self,
        model: Option<&str>,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        let protocol = self.control_protocol.as_ref().ok_or_else(|| {
            ClaudeAgentError::ControlProtocol("Control protocol not initialized".to_string())
        })?;
        protocol.set_model(model).await
    }

    /// Rewind files to a specific user message checkpoint.
    pub async fn rewind_files(
        &self,
        user_message_id: &str,
    ) -> Result<ControlResponse, ClaudeAgentError> {
        let protocol = self.control_protocol.as_ref().ok_or_else(|| {
            ClaudeAgentError::ControlProtocol("Control protocol not initialized".to_string())
        })?;
        protocol.rewind_files(user_message_id).await
    }

    /// Disconnect from Claude Code CLI.
    pub async fn disconnect(&mut self) -> Result<(), ClaudeAgentError> {
        if let Some(ref mut transport) = self.transport {
            transport.close().await?;
        }
        self.transport = None;

        if let Some(session) = self.session_manager.current_session_mut() {
            session.deactivate();
        }

        Ok(())
    }

    /// Get the current session.
    pub fn current_session(&self) -> Option<&Session> {
        self.session_manager.current_session()
    }

    /// Get a reference to the hook registry.
    pub fn hook_registry(&self) -> &HookRegistry {
        &self.hook_registry
    }

    /// Get a mutable reference to the hook registry.
    pub fn hook_registry_mut(&mut self) -> &mut HookRegistry {
        &mut self.hook_registry
    }

    /// Get a reference to the MCP manager.
    pub fn mcp_manager(&self) -> &McpServerManager {
        &self.mcp_manager
    }

    /// Get a mutable reference to the MCP manager.
    pub fn mcp_manager_mut(&mut self) -> &mut McpServerManager {
        &mut self.mcp_manager
    }
}
