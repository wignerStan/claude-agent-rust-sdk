//! Transport layer for Claude Agent SDK.
//!
//! This module provides the transport abstraction for communicating with Claude Code CLI.
//! The transport layer handles:
//! - Process spawning and lifecycle management
//! - Bidirectional communication (stdin/stdout)
//! - Streaming JSON message parsing
//! - Error handling and recovery
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
//!     // Connect to the transport
//!     Transport::connect(&mut transport).await?;
//!     transport.write("What is 2+2?").await?;
//!
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
//!     transport.close().await?;
//!     Ok(())
//! }
//! ```

pub mod parser;
pub mod reader;
pub mod subprocess;

use async_trait::async_trait;
use claude_agent_types::ClaudeAgentError;
use futures::stream::BoxStream;

pub use subprocess::SubprocessTransport;

/// Transport trait for communication with Claude Code.
///
/// This trait defines the interface for all transport implementations, providing
/// a common abstraction for different communication methods (subprocess, HTTP, SSE, etc.).
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to allow safe concurrent access
/// from multiple threads or async tasks.
///
/// # Error Handling
///
/// All methods return `Result<T, ClaudeAgentError>` to ensure proper error
/// propagation throughout the application. Implementations should convert
/// transport-specific errors to `ClaudeAgentError` using the appropriate variant.
///
/// # Lifecycle
///
/// 1. **Connect**: Establish connection and initialize resources
/// 2. **Write/Read**: Bidirectional communication
/// 3. **Close**: Clean up resources and release handles
///
/// # Example Implementation
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use futures::stream::BoxStream;
/// use claude_agent_types::ClaudeAgentError;
///
/// #[async_trait]
/// impl Transport for MyTransport {
///     async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
///         // Initialize connection, spawn processes, etc.
///         Ok(())
///     }
///
///     async fn write(&self, data: &str) -> Result<(), ClaudeAgentError> {
///         // Send data to transport
///         Ok(())
///     }
///
///     async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
///         // Return stream of incoming messages
///         Box::pin(futures::stream::empty())
///     }
///
///     async fn close(&mut self) -> Result<(), ClaudeAgentError> {
///         // Clean up resources
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the transport.
    ///
    /// This method establishes the connection and initializes all necessary resources.
    /// Implementations should:
    /// - Spawn processes or establish network connections
    /// - Initialize buffers and channels
    /// - Validate connection parameters
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::CLIConnection` if the connection cannot be established,
    /// or other transport-specific errors.
    ///
    /// # Thread Safety
    ///
    /// This method may be called from any thread, so implementations must ensure
    /// proper synchronization of internal state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
    ///     self.process = Some(Command::new("claude").spawn()?);
    ///     Ok(())
    /// }
    /// ```
    async fn connect(&mut self) -> Result<(), ClaudeAgentError>;

    /// Write data to the transport.
    ///
    /// Sends a message to the transport. The data is sent as a UTF-8 string
    /// followed by a newline character.
    ///
    /// # Parameters
    ///
    /// - `data`: The message content to send. Must be valid UTF-8.
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Transport` if the write fails, or other
    /// transport-specific errors.
    ///
    /// # Buffering
    ///
    /// Implementations may buffer writes for efficiency, but must ensure data is
    /// eventually flushed to the underlying transport.
    ///
    /// # Thread Safety
    ///
    /// This method may be called concurrently from multiple tasks. Implementations must
    /// use appropriate synchronization (e.g., `Mutex`, `Arc`).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn write(&self, data: &str) -> Result<(), ClaudeAgentError> {
    ///     let mut stdin = self.stdin.lock().await;
    ///     stdin.write_all(data.as_bytes()).await?;
    ///     stdin.write_all(b"\n").await?;
    ///     stdin.flush().await?;
    ///     Ok(())
    /// }
    /// ```
    async fn write(&self, data: &str) -> Result<(), ClaudeAgentError>;

    /// Read messages from the transport as a stream.
    ///
    /// Returns a stream of incoming messages. Each message is parsed from the
    /// transport's raw data (e.g., JSON from stdout).
    ///
    /// # Stream Semantics
    ///
    /// - The stream yields `Result<serde_json::Value, ClaudeAgentError>` for each message
    /// - The stream may be infinite until the transport is closed
    /// - Implementations should handle backpressure appropriately
    ///
    /// # Lifetime
    ///
    /// The returned stream is tied to `&self`, so it cannot outlive the transport.
    /// This ensures proper resource cleanup when the transport is dropped.
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Transport` for read errors, or
    /// `ClaudeAgentError::JSONDecode` for parsing errors.
    ///
    /// # Thread Safety
    ///
    /// The returned stream must be safe to use from multiple concurrent tasks.
    /// Implementations should use channels or other synchronization primitives.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
    ///     let rx = self.message_channel.subscribe();
    ///     Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
    /// }
    /// ```
    async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>>;

    /// Close the transport.
    ///
    /// Releases all resources associated with the transport. Implementations should:
    /// - Flush any buffered data
    /// - Close file handles and network connections
    /// - Abort background tasks
    /// - Wait for processes to exit gracefully
    ///
    /// # Errors
    ///
    /// Returns `ClaudeAgentError::Transport` if cleanup fails, or other
    /// transport-specific errors.
    ///
    /// # Thread Safety
    ///
    /// This method should be safe to call from any thread, even if other methods
    /// are currently in use.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn close(&mut self) -> Result<(), ClaudeAgentError> {
    ///     if let Some(abort_handle) = self.reader_task.take() {
    ///         abort_handle.abort();
    ///     }
    ///     if let Some(mut process) = self.process.take() {
    ///         process.kill().await?;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn close(&mut self) -> Result<(), ClaudeAgentError>;
}
