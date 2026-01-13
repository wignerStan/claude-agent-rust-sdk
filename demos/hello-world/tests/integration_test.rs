//! Tests for Hello World demo.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::ClaudeAgentOptions;

#[tokio::test]
async fn test_hello_world_basic_query() {
    let response = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [{"type": "text", "text": "Hello! I'm Claude, an AI assistant."}],
            "role": "assistant",
            "model": "claude-opus-4-1-20250805"
        }
    });

    let mock_transport = MockTransport::new(vec![response]);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let stream = client.query("Hello, Claude!").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 1);
            match &msg.content[0] {
                ContentBlock::Text(text_block) => {
                    assert!(text_block.text.contains("Claude"));
                }
                _ => panic!("Expected text content"),
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[tokio::test]
async fn test_hello_world_with_options() {
    let response = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [{"type": "text", "text": "Response"}],
            "role": "assistant",
            "model": "claude-opus-4-1-20250805"
        }
    });

    let mock_transport = MockTransport::new(vec![response]);
    let options = ClaudeAgentOptions {
        max_turns: Some(100),
        model: Some("claude-opus-4-1-20250805".to_string()),
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let stream = client.query("Test").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_hello_world_multiple_messages() {
    let response1 = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [{"type": "text", "text": "First message"}],
            "role": "assistant",
            "model": "claude-opus-4-1-20250805"
        }
    });

    let response2 = serde_json::json!({
        "type": "result",
        "subtype": "success",
        "duration_ms": 100,
        "duration_api_ms": 50,
        "is_error": false,
        "num_turns": 1,
        "session_id": "test-session",
        "result": "Query completed"
    });

    let mock_transport = MockTransport::new(vec![response1, response2]);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let stream = client.query("Test").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 2);
    assert!(matches!(messages[0], Message::Assistant(_)));
    assert!(matches!(messages[1], Message::Result(_)));
}

#[tokio::test]
async fn test_hello_world_error_handling() {
    let error_response = serde_json::json!({
        "type": "error",
        "error": {
            "type": "connection_error",
            "message": "Failed to connect"
        }
    });

    let mock_transport = MockTransport::new(vec![error_response]);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let stream = client.query("Test").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
    assert!(matches!(messages[0], Message::Error(_)));
}

#[tokio::test]
async fn test_hello_world_disconnect() {
    let response = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [{"type": "text", "text": "Response"}],
            "role": "assistant",
            "model": "claude-opus-4-1-20250805"
        }
    });

    let mock_transport = MockTransport::new(vec![response]);
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let stream = client.query("Test").await.unwrap();

    let _ = StreamCollector::collect(stream).await.unwrap();

    client.disconnect().await.unwrap();
}
