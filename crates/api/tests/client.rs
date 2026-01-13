use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use serde_json::json;
use std::sync::{Arc, Mutex};

use claude_agent_api::ClaudeAgentClient;
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
        // Clone responses to return a stream
        let responses = self.responses.clone();
        Box::pin(stream::iter(responses.into_iter().map(Ok)))
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_client_query_single_prompt() {
    let response = json!({
        "type": "assistant",
        "message": {
            "content": [{"type": "text", "text": "4"}],
            "role": "assistant",
            "model": "claude-test"
        }
    });

    let mock_transport = MockTransport::new(vec![response]);
    let sent_data = mock_transport.sent_data.clone();

    let options = ClaudeAgentOptions::default();
    let mut client = ClaudeAgentClient::new(Some(options));

    // Inject mock transport
    client.set_transport(Box::new(mock_transport));

    // Perform query
    use futures::StreamExt;
    let mut stream = client.query("What is 2+2?").await.expect("Query failed");

    let mut messages = Vec::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => messages.push(msg),
            Err(e) => panic!("Stream error: {}", e),
        }
    }

    // In our mock, we return 1 message.
    // ClaudeAgent::query reads messages until stream ends or result message.
    // Our mock stream ends after 1 message.

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 1);
            match &msg.content[0] {
                ContentBlock::Text(text_block) => assert_eq!(text_block.text, "4"),
                _ => panic!("Expected text content"),
            }
        }
        _ => panic!("Expected AssistantMessage"),
    }

    let sent = sent_data.lock().unwrap();
    assert!(!sent.is_empty());
}

#[tokio::test]
async fn test_client_control_methods() {
    let mock_transport = MockTransport::new(vec![]); // No responses needed for simple writes
    let sent_data = mock_transport.sent_data.clone();

    let mut client = ClaudeAgentClient::new(None);
    client.set_transport(Box::new(mock_transport));
    client.connect().await.expect("Connect failed");

    // Test Interrupt
    client.interrupt().await.expect("Interrupt failed");
    {
        let sent = sent_data.lock().unwrap();
        let last_msg = sent.last().unwrap();
        let json: serde_json::Value = serde_json::from_str(last_msg).unwrap();
        assert_eq!(json["type"], "control_request");
        assert_eq!(json["request"]["subtype"], "interrupt");
    }

    // Test Set Model
    client
        .set_model(Some("claude-test"))
        .await
        .expect("Set model failed");
    {
        let sent = sent_data.lock().unwrap();
        let last_msg = sent.last().unwrap();
        let json: serde_json::Value = serde_json::from_str(last_msg).unwrap();
        assert_eq!(json["type"], "control_request");
        assert_eq!(json["request"]["subtype"], "set_model");
        assert_eq!(json["request"]["model"], "claude-test");
    }

    // Test Set Permission Mode
    client
        .set_permission_mode("plan")
        .await
        .expect("Set permission mode failed");
    {
        let sent = sent_data.lock().unwrap();
        let last_msg = sent.last().unwrap();
        let json: serde_json::Value = serde_json::from_str(last_msg).unwrap();
        assert_eq!(json["type"], "control_request");
        assert_eq!(json["request"]["subtype"], "set_permission_mode");
        assert_eq!(json["request"]["mode"], "plan");
    }
}
