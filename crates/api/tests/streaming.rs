//! Streaming client tests.

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use serde_json::json;
use std::sync::{Arc, Mutex};

use claude_agent_api::ClaudeAgentClient;
use claude_agent_transport::Transport;
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions};

struct MockTransport {
    sent_data: Arc<Mutex<Vec<String>>>,
    responses: Vec<serde_json::Value>,
}

impl MockTransport {
    fn new(responses: Vec<serde_json::Value>) -> Self {
        Self { sent_data: Arc::new(Mutex::new(Vec::new())), responses }
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

#[tokio::test]
async fn test_streaming_multiple_messages() {
    let responses = vec![
        json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "Part 1"}]}}),
        json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "Part 2"}]}}),
        json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "Part 3"}]}}),
    ];

    let mock = MockTransport::new(responses);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock));

    let mut stream = client.query("Stream test").await.expect("Query failed");
    let mut messages = Vec::new();
    while let Some(res) = stream.next().await {
        messages.push(res.expect("Stream error"));
    }

    assert_eq!(messages.len(), 3);
    for (i, msg) in messages.iter().enumerate() {
        if let Message::Assistant(m) = msg {
            if let ContentBlock::Text(t) = &m.content[0] {
                assert_eq!(t.text, format!("Part {}", i + 1));
            }
        }
    }
}

#[tokio::test]
async fn test_streaming_empty_response() {
    let mock = MockTransport::new(vec![]);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock));

    let mut stream = client.query("Empty test").await.expect("Query failed");
    let mut count = 0;
    while (stream.next().await).is_some() {
        count += 1;
    }
    assert_eq!(count, 0);
}
