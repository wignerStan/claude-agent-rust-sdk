use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::json;
use std::sync::{Arc, Mutex};

use claude_agent_api::ClaudeAgentClient;
use claude_agent_transport::Transport;
use claude_agent_types::message::ContentBlock;
use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions, Message};

// Reusable MockTransport
struct MockTransport {
    pub sent_data: Arc<Mutex<Vec<String>>>,
    pub responses: Vec<serde_json::Value>,
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
async fn test_simple_query_response() {
    let response1 = json!({
        "type": "assistant",
        "message": {
            "role": "assistant",
            "content": [{"type": "text", "text": "2 + 2 equals 4"}],
            "model": "claude-opus-4-1-20250805"
        }
    });

    let response2 = json!({
        "type": "result",
        "subtype": "success",
        "duration_ms": 1000,
        "duration_api_ms": 800,
        "is_error": false,
        "num_turns": 1,
        "session_id": "test-session",
        "total_cost_usd": 0.001
    });

    let mock_transport = MockTransport::new(vec![response1, response2]);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock_transport));

    let mut stream = client.query("What is 2 + 2?").await.expect("Query failed");
    let mut messages = Vec::new();
    while let Some(msg) = stream.next().await {
        messages.push(msg.expect("Message error"));
    }

    assert_eq!(messages.len(), 2);

    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 1);
            match &msg.content[0] {
                ContentBlock::Text(t) => assert_eq!(t.text, "2 + 2 equals 4"),
                _ => panic!("Expected text"),
            }
        },
        _ => panic!("Expected Assistant message"),
    }

    match &messages[1] {
        Message::Result(msg) => {
            assert_eq!(msg.total_cost_usd, Some(0.001));
            assert_eq!(msg.session_id, "test-session");
        },
        _ => panic!("Expected Result message"),
    }
}

#[tokio::test]
async fn test_query_with_tool_use() {
    let response1 = json!({
        "type": "assistant",
        "message": {
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Let me read that file for you.",
                },
                {
                    "type": "tool_use",
                    "id": "tool-123",
                    "name": "Read",
                    "input": {"file_path": "/test.txt"},
                },
            ],
            "model": "claude-opus-4-1-20250805",
        },
    });

    let response2 = json!({
         "type": "result",
         "subtype": "success",
         "duration_ms": 1500,
         "duration_api_ms": 1200,
         "is_error": false,
         "num_turns": 1,
         "session_id": "test-session-2",
         "total_cost_usd": 0.002
    });

    let mock_transport = MockTransport::new(vec![response1, response2]);
    let options =
        ClaudeAgentOptions { allowed_tools: vec!["Read".to_string()], ..Default::default() };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.set_transport(Box::new(mock_transport));

    let mut stream = client.query("Read /test.txt").await.expect("Query failed");
    let mut messages = Vec::new();
    while let Some(msg) = stream.next().await {
        messages.push(msg.expect("Message error"));
    }

    assert_eq!(messages.len(), 2);

    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 2);
            match &msg.content[0] {
                ContentBlock::Text(t) => assert_eq!(t.text, "Let me read that file for you."),
                _ => panic!("Expected text"),
            }
            match &msg.content[1] {
                ContentBlock::ToolUse(t) => {
                    assert_eq!(t.name, "Read");
                    assert_eq!(t.input["file_path"], "/test.txt");
                },
                _ => panic!("Expected tool use"),
            }
        },
        _ => panic!("Expected Assistant message"),
    }
}
