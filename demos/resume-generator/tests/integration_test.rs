//! Tests for Resume Generator demo.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::{config::SystemPromptConfig, ClaudeAgentOptions, Message};
use claude_agent_types::ClaudeAgentError;
use std::time::Duration;
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_resume_generation_basic() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Researching Jane Doe..."},
                    {"type": "tool_use", "id": "call_1", "name": "WebSearch", "input": {"query": "Jane Doe"}},
                    {"type": "tool_use", "id": "call_2", "name": "Write", "input": {"file_path": "resume.docx"}},
                    {"type": "text", "text": "I've created a professional resume for Jane Doe."}
                ],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions {
            allowed_tools: vec![
                "WebSearch".to_string(),
                "WebFetch".to_string(),
                "Write".to_string(),
                "Read".to_string(),
            ],
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client
            .query("Research Jane Doe and create a resume.")
            .await
            .unwrap();

        let messages: Vec<_> = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        if let Message::Assistant(msg) = &messages[0] {
            assert!(msg.content.len() >= 3);
        } else {
            panic!("Expected Assistant message");
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_resume_with_focus_areas() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Focusing on leadership and management skills..."},
                    {"type": "tool_use", "id": "call_1", "name": "WebSearch", "input": {"query": "John Smith leadership"}},
                    {"type": "tool_use", "id": "call_2", "name": "Write", "input": {"file_path": "resume.docx"}},
                    {"type": "text", "text": "Created resume highlighting John's leadership experience."}
                ],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions {
            allowed_tools: vec![
                "WebSearch".to_string(),
                "WebFetch".to_string(),
                "Write".to_string(),
            ],
            system_prompt: Some(SystemPromptConfig::Text(
                "Focus on leadership and management skills. Use strong action verbs and measurable achievements.".to_string()
            )),
            max_turns: Some(25),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client
            .query("Research John Smith and create a resume focusing on leadership and management.")
            .await
            .unwrap();

        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_tool_whitelisting() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "I'll create a resume for Alice."}
                ],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions {
            allowed_tools: vec!["WebSearch".to_string(), "Write".to_string()],
            disallowed_tools: vec!["Bash".to_string(), "Grep".to_string()],
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Create a resume for Alice.").await.unwrap();

        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_max_turns_enforcement() {
    timeout(TEST_TIMEOUT, async {
        let response1 = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Researching..."}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });
        let response2 = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Still researching..."}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });
        let response3 = serde_json::json!({
            "type": "result",
            "subtype": "max_turns_reached",
            "duration_ms": 100,
            "duration_api_ms": 50,
            "is_error": true,
            "num_turns": 3,
            "session_id": "test-session",
            "result": "Maximum turns reached"
        });

        let mock_transport = MockTransport::new(vec![response1, response2, response3]);
        let options = ClaudeAgentOptions {
            max_turns: Some(3),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Research Bob Johnson.").await.unwrap();

        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 3);
        assert!(matches!(messages[2], Message::Result(_)));
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_system_prompt_application() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Creating professional resume with clear sections..."}
                ],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions {
            system_prompt: Some(SystemPromptConfig::Text(
                "You are a professional resume writer. Create a 1-page resume in .docx format."
                    .to_string(),
            )),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Create a resume for Alice.").await.unwrap();

        let messages: Vec<_> = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
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

        let stream = client.query("Create a resume.").await.unwrap();

        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], Message::Error(_)));
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_disconnect_cleanup() {
    timeout(TEST_TIMEOUT, async {
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Resume created successfully."}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let options = ClaudeAgentOptions::default();

        let mut client = ClaudeAgentClient::new(Some(options));
        client.set_transport(Box::new(mock_transport));

        client.connect().await.unwrap();

        let stream = client.query("Create a resume.").await.unwrap();

        let _ = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

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
                "content": [{"type": "text", "text": "Using Sonnet model as requested."}],
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

        let stream = client.query("Create a resume.").await.unwrap();

        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        if let Message::Assistant(msg) = &messages[0] {
            assert_eq!(msg.model, "claude-sonnet-4-5");
        } else {
            panic!("Expected Assistant message");
        }
    })
    .await
    .expect("Test timed out");
}
