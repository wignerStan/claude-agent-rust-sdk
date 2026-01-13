//! Tests for Hello World V2 demo.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::ClaudeAgentOptions;
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_basic_session() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Hello! I'm Claude, an AI assistant."}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions {
            model: Some("claude-sonnet-4-5".to_string()),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Hello! My name is Alice.").await.unwrap();

        let messages: Vec<_> = StreamCollector::collect(stream).await.unwrap();

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
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_multi_turn_conversation() {
    timeout(TEST_TIMEOUT, async {
        let responses = vec![
            serde_json::json!({
                "type": "assistant",
                "message": {
                    "content": [{"type": "text", "text": "Rust is a systems programming language."}],
                    "role": "assistant",
                    "model": "claude-sonnet-4-5"
                }
            }),
            serde_json::json!({
                "type": "assistant",
                "message": {
                    "content": [{"type": "text", "text": "Key features include ownership and borrowing."}],
                    "role": "assistant",
                    "model": "claude-sonnet-4-5"
                }
            }),
            serde_json::json!({
                "type": "assistant",
                "message": {
                    "content": [{"type": "text", "text": "It uses a borrow checker for memory safety."}],
                    "role": "assistant",
                    "model": "claude-sonnet-4-5"
                }
            }),
        ];

        let mock_transport = MockTransport::new(responses);
        let options = ClaudeAgentOptions {
            model: Some("claude-sonnet-4-5".to_string()),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let prompts = vec![
            "I'm learning Rust programming.",
            "What are the key features of Rust?",
            "Can you explain ownership and borrowing?",
        ];

        for prompt in prompts {
            let mut stream = client.query(prompt).await.unwrap();

            use futures::StreamExt;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_session_lifecycle() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Session started"}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions::default();

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Hello!").await.unwrap();

        let _: Vec<_> = StreamCollector::collect(stream).await.unwrap();

        client.disconnect().await.unwrap();
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_one_shot_prompt() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Paris"}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);

        let prompt = "What is the capital of France?";

        // Fix: Use ClaudeAgentClient to allow mocking transport for one-shot test
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(mock_transport));
        client.connect().await.unwrap();
        let stream = client.query(prompt).await.unwrap();

        let messages = StreamCollector::collect(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        match &messages[0] {
            Message::Assistant(msg) => match &msg.content[0] {
                ContentBlock::Text(text_block) => {
                    assert!(text_block.text.contains("Paris"));
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected Assistant message"),
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_error_handling() {
    timeout(TEST_TIMEOUT, async {
        let error_response = serde_json::json!({
            "type": "error",
            "error": {
                "type": "connection_error",
                "message": "Failed to connect"
            }
        });

        let mock_transport = MockTransport::new(vec![error_response]);
        let options = ClaudeAgentOptions::default();

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Hello!").await.unwrap();

        let messages = StreamCollector::collect(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], Message::Error(_)));
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_disconnect() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Response"}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions::default();

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Hello!").await.unwrap();

        let _: Vec<_> = StreamCollector::collect(stream).await.unwrap();

        client.disconnect().await.unwrap();
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_model_selection() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Using Sonnet model"}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions {
            model: Some("claude-sonnet-4-5".to_string()),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Hello!").await.unwrap();

        let messages = StreamCollector::collect(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        match &messages[0] {
            Message::Assistant(msg) => {
                assert_eq!(msg.model, "claude-sonnet-4-5");
            }
            _ => panic!("Expected Assistant message"),
        }
    })
    .await
    .expect("Test timed out");
}
