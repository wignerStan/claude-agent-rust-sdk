//! Testing utilities and mock implementations for demo testing.
//!
//! Provides:
//! - `MockTransport`: Configurable mock transport for testing
//! - `StreamCollector`: Helper for collecting stream results
//! - `TestFixture`: Common test setup and teardown

use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use claude_agent_transport::Transport;
use claude_agent_types::ClaudeAgentError;

/// Mock transport implementation for testing SDK clients.
///
/// Allows configurable responses, error injection, and state tracking.
pub struct MockTransport {
    /// All data sent through write() is stored here for assertions
    pub sent_data: Arc<Mutex<Vec<String>>>,
    /// Responses to return from read_messages()
    pub responses: Vec<serde_json::Value>,
    /// Optional delay before returning responses (for testing timeouts)
    pub response_delay: Option<Duration>,
    /// Optional error to inject on next read
    pub inject_error: Option<ClaudeAgentError>,
}

impl MockTransport {
    /// Create a new mock transport with given responses.
    pub fn new(responses: Vec<serde_json::Value>) -> Self {
        Self {
            sent_data: Arc::new(Mutex::new(Vec::new())),
            responses,
            response_delay: None,
            inject_error: None,
        }
    }

    /// Create a mock transport that returns a single assistant message.
    pub fn with_text_response(text: &str) -> Self {
        let response = json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": text}],
                "role": "assistant",
                "model": "claude-test"
            }
        });
        Self::new(vec![response])
    }

    /// Create a mock transport that returns a result message.
    pub fn with_result_message() -> Self {
        let response = json!({
            "type": "result",
            "result": {
                "type": "success",
                "message": "Query completed"
            }
        });
        Self::new(vec![response])
    }

    /// Set a delay before returning responses.
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = Some(delay);
        self
    }

    /// Inject an error on next read.
    pub fn with_error(mut self, error: ClaudeAgentError) -> Self {
        self.inject_error = Some(error);
        self
    }

    /// Get all data sent through write() for assertions.
    pub fn get_sent_data(&self) -> Vec<String> {
        self.sent_data.lock().unwrap().clone()
    }

    /// Clear sent data buffer.
    pub fn clear_sent_data(&self) {
        self.sent_data.lock().unwrap().clear();
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }

    async fn write(&self, data: &str) -> Result<(), ClaudeAgentError> {
        self.sent_data.lock().unwrap().push(data.to_string());
        Ok(())
    }

    async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
        if let Some(delay) = self.response_delay {
            tokio::time::sleep(delay).await;
        }

        if let Some(error) = &self.inject_error {
            let error = error.clone();
            return Box::pin(stream::iter(vec![Err(error)]));
        }

        let responses = self.responses.clone();
        Box::pin(stream::iter(responses.into_iter().map(Ok)))
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }
}

/// Helper for collecting all items from a stream into a Vec.
pub struct StreamCollector;

impl StreamCollector {
    /// Collect all items from a stream, returning errors if any occur.
    pub async fn collect<T, E>(
        stream: impl futures::Stream<Item = Result<T, E>>,
    ) -> Result<Vec<T>, E>
    where
        T: Send,
        E: Send,
    {
        stream
            .map(|result| result)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
    }

    /// Collect all successful items, ignoring errors.
    pub async fn collect_ok<T, E>(stream: impl futures::Stream<Item = Result<T, E>>) -> Vec<T>
    where
        T: Send,
        E: std::fmt::Debug,
    {
        stream
            .filter_map(|result| async {
                match result {
                    Ok(item) => Some(item),
                    Err(e) => {
                        eprintln!("Stream error (ignored): {:?}", e);
                        None
                    }
                }
            })
            .collect()
            .await
    }
}

/// Common test fixture setup and teardown.
pub struct TestFixture {
    /// Mock transport for testing
    pub transport: MockTransport,
    /// Temporary directory for test files
    pub temp_dir: tempfile::TempDir,
}

impl TestFixture {
    /// Create a new test fixture with a mock transport.
    pub fn new(responses: Vec<serde_json::Value>) -> Self {
        Self {
            transport: MockTransport::new(responses),
            temp_dir: tempfile::TempDir::new().expect("Failed to create temp dir"),
        }
    }

    /// Create a test fixture with a simple text response.
    pub fn with_text(text: &str) -> Self {
        Self::new(vec![json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": text}],
                "role": "assistant",
                "model": "claude-test"
            }
        })])
    }

    /// Get path to temporary directory.
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a file in the temporary directory.
    pub fn create_file(&self, name: &str, content: &str) -> std::path::PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::write(&path, content).expect("Failed to write test file");
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_transport_basic() {
        let response = json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Hello"}],
                "role": "assistant",
                "model": "claude-test"
            }
        });

        let mut transport = MockTransport::new(vec![response]);
        transport.connect().await.unwrap();
        transport.write("test data").await.unwrap();

        let sent = transport.get_sent_data();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], "test data");
    }

    #[tokio::test]
    async fn test_stream_collector() {
        let stream = stream::iter(vec![
            Ok::<_, ClaudeAgentError>(serde_json::json!({
                "type": "assistant",
                "message": {
                    "content": [{"type": "text", "text": "Hello"}],
                    "role": "assistant",
                    "model": "claude-test"
                }
            })),
            Ok(serde_json::json!({
                "type": "result",
                "result": {
                    "type": "success",
                    "message": "Done"
                }
            })),
        ]);

        let messages = StreamCollector::collect(stream).await.unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn test_stream_collector_ok() {
        let stream = stream::iter(vec![
            Ok::<_, ClaudeAgentError>(serde_json::json!({
                "type": "assistant",
                "message": {
                    "content": [{"type": "text", "text": "Hello"}],
                    "role": "assistant",
                    "model": "claude-test"
                }
            })),
            Err(ClaudeAgentError::CLIConnection("Test error".to_string())),
        ]);

        let messages = StreamCollector::collect_ok(stream).await;
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = TestFixture::with_text("test");
        assert!(fixture.temp_path().exists());
        let file_path = fixture.create_file("test.txt", "content");
        assert!(file_path.exists());
        assert_eq!(std::fs::read_to_string(file_path).unwrap(), "content");
    }
}
