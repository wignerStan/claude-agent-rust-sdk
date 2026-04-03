//! Shared test helpers for API crate integration tests.
//!
//! Each integration test binary compiles its own copy of this module,
//! so individual items may appear unused when viewed from a single test.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use claude_agent::transport::Transport;
use claude_agent::types::message::Message;
use claude_agent::types::ClaudeAgentError;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::json;

// --- Mock Transport ---

/// A mock transport that pre-loads responses and tracks sent data.
pub struct MockTransport {
    pub sent_data: Arc<Mutex<Vec<String>>>,
    responses: Vec<serde_json::Value>,
}

impl MockTransport {
    pub fn new(responses: Vec<serde_json::Value>) -> Self {
        Self { sent_data: Arc::new(Mutex::new(Vec::new())), responses }
    }

    pub fn sent_data_clone(&self) -> Arc<Mutex<Vec<String>>> {
        self.sent_data.clone()
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
        let responses = self.responses.clone();
        Box::pin(stream::iter(responses.into_iter().map(Ok)))
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }
}

// --- Control Reply Transport ---

/// A mock transport that auto-replies to control_request messages.
pub struct ControlReplyTransport {
    tx: tokio::sync::broadcast::Sender<Result<serde_json::Value, ClaudeAgentError>>,
    response_body: serde_json::Value,
}

impl ControlReplyTransport {
    pub fn new(response_body: serde_json::Value) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(100);
        Self { tx, response_body }
    }
}

#[async_trait]
impl Transport for ControlReplyTransport {
    async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }

    async fn write(&self, data: &str) -> Result<(), ClaudeAgentError> {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
            if val.get("type").and_then(|s| s.as_str()) == Some("control_request") {
                let req_id = val.get("request_id").and_then(|s| s.as_str()).unwrap_or("");
                let resp = json!({
                    "type": "control_response",
                    "request_id": req_id,
                    "success": true,
                    "response": self.response_body
                });
                let _ = self.tx.send(Ok(resp));
            }
        }
        Ok(())
    }

    async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
        let rx = self.tx.subscribe();
        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(res) => return Some((res, rx)),
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    Err(_) => continue,
                }
            }
        }))
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }
}

// --- Helpers ---

/// Create a connected client with mock responses, returning client and sent data tracker.
pub async fn connected_client(
    responses: Vec<serde_json::Value>,
) -> (claude_agent::api::ClaudeAgentClient, Arc<Mutex<Vec<String>>>) {
    let mock = MockTransport::new(responses);
    let sent_data = mock.sent_data_clone();
    let mut client = claude_agent::api::ClaudeAgentClient::new(Some(
        claude_agent::types::ClaudeAgentOptions::default(),
    ));
    client.set_transport(Box::new(mock));
    client.connect().await.unwrap();
    (client, sent_data)
}

/// Collect all messages from a stream.
pub async fn collect_messages(
    stream: BoxStream<'_, Result<Message, ClaudeAgentError>>,
) -> Vec<Message> {
    let mut messages = Vec::new();
    futures::pin_mut!(stream);
    while let Some(result) = stream.next().await {
        messages.push(result.expect("Stream item error"));
    }
    messages
}
