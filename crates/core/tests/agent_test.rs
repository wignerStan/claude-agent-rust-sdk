use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use serde_json::json;
use std::sync::{Arc, Mutex};

use claude_agent_core::agent::ClaudeAgent;
use claude_agent_transport::Transport;
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions};

// Mock Transport
struct MockTransport {
    pub sent_data: Arc<Mutex<Vec<String>>>,
    pub responses: Vec<serde_json::Value>,
}

impl MockTransport {
    fn new(responses: Vec<serde_json::Value>) -> Self {
        Self {
            sent_data: Arc::new(Mutex::new(Vec::new())),
            responses,
        }
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
async fn test_agent_control_flow() {
    // 1. Setup Mock messages
    // First message: Control Request (e.g. set_model) - Agent receives this, handles it, writes response.
    // Second message: Normal Assistant Message - Agent yields this.

    let control_req = json!({
        "type": "control_request",
        "request_id": "req-1",
        "request": {
            "subtype": "set_model",
            "model": "claude-3.5-sonnet"
        }
    });

    let assistant_msg = json!({
        "type": "assistant",
        "message": {
             "role": "assistant",
             "content": [{"type": "text", "text": "Hello"}]
        }
    });

    let mock_transport = MockTransport::new(vec![control_req, assistant_msg]);
    let sent_data = mock_transport.sent_data.clone();

    // 2. Init Agent
    let options = ClaudeAgentOptions::default();
    let mut agent = ClaudeAgent::new(options);
    agent.set_transport(Box::new(mock_transport));

    // 3. Run Query
    let mut stream = agent
        .query("test prompt")
        .await
        .expect("Failed to start query");

    // 4. Consume stream
    let mut messages = Vec::new();
    while let Some(res) = stream.next().await {
        messages.push(res.expect("Stream error"));
    }

    // 5. Verify results
    // We expect 1 yielded message (the assistant one)
    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(m) => {
            if let ContentBlock::Text(t) = &m.content[0] {
                assert_eq!(t.text, "Hello");
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected Assistant message"),
    }

    // 6. Verify sent data
    // Expect:
    // 1. JSON serialized "test prompt" (initial write)
    // 2. Control Response (for set_model)
    let writes = sent_data.lock().unwrap();
    assert_eq!(writes.len(), 2);

    // Check first write is valid JSON and contains "test prompt"
    let initial_msg: serde_json::Value =
        serde_json::from_str(&writes[0]).expect("First write should be JSON");
    assert_eq!(initial_msg["type"], "user");
    assert_eq!(initial_msg["message"]["role"], "user");
    assert_eq!(initial_msg["message"]["content"][0]["text"], "test prompt");

    // Verify control response
    let response_json: serde_json::Value =
        serde_json::from_str(&writes[1]).expect("Invalid JSON response");
    assert_eq!(response_json["type"], "control_response");
    assert_eq!(response_json["response"]["request_id"], "req-1");
    // status: not_implemented for set_model currently
    assert_eq!(
        response_json["response"]["response"]["status"],
        "not_implemented"
    );
}
