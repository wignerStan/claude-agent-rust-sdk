//! Tests for Resume Generator demo.

use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::ClaudeAgentOptions;
use claude_agent_types::message::{ContentBlock, Message};
use futures::StreamExt;

#[tokio::test]
async fn test_resume_generation_basic() {
    let response = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                {"type": "text", "text": "Researching Jane Doe..."},
                {"type": "tool_use", "name": "WebSearch", "input": {"query": "Jane Doe"}},
                {"type": "tool_use", "name": "Write", "input": {"file_path": "resume.docx"}},
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

    let mut stream = client.query("Research Jane Doe and create a resume.").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert!(msg.content.len() >= 3);
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[tokio::test]
async fn test_resume_with_focus_areas() {
    let response = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                {"type": "text", "text": "Focusing on leadership and management skills..."},
                {"type": "tool_use", "name": "WebSearch", "input": {"query": "John Smith leadership"}},
                {"type": "tool_use", "name": "Write", "input": {"file_path": "resume.docx"}},
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
        system_prompt: Some(
            "Focus on leadership and management skills. Use strong action verbs and measurable achievements.".to_string()
        ),
        max_turns: Some(25),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let mut stream = client
        .query("Research John Smith and create a resume focusing on leadership and management.")
        .await
        .unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_tool_whitelisting() {
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
        allowed_tools: vec![
            "WebSearch".to_string(),
            "Write".to_string(),
        ],
        disallowed_tools: vec!["Bash".to_string(), "Grep".to_string()],
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let mut stream = client.query("Create a resume for Alice.").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_max_turns_enforcement() {
    let responses = vec![
        serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Researching..."}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        }),
        serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Still researching..."}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        }),
        serde_json::json!({
            "type": "result",
            "result": {
                "type": "max_turns_reached",
                "message": "Maximum turns reached"
            }
        }),
    ];

    let mock_transport = MockTransport::new(responses);
    let options = ClaudeAgentOptions {
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let mut stream = client.query("Research Bob Johnson.").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 4);
    assert!(matches!(messages[3], Message::Result(_)));
}

#[tokio::test]
async fn test_system_prompt_application() {
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
        system_prompt: Some(
            "You are a professional resume writer. Create a 1-page resume in .docx format.".to_string()
        ),
        ..Default::default()
    };

    let mut client = ClaudeAgentClient::new(Some(options));
    client.set_transport(Box::new(mock_transport));

    client.connect().await.unwrap();

    let mut stream = client.query("Create a resume for Alice.").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_error_handling() {
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

    let mut stream = client.query("Create a resume.").await.unwrap();

    let messages = StreamCollector::collect(stream).await;

    assert!(messages.is_err());
}

#[tokio::test]
async fn test_disconnect_cleanup() {
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

    let mut stream = client.query("Create a resume.").await.unwrap();

    let _ = StreamCollector::collect(stream).await.unwrap();

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_model_selection() {
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

    let mut stream = client.query("Create a resume.").await.unwrap();

    let messages = StreamCollector::collect(stream).await.unwrap();

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.model, "claude-sonnet-4-5");
        }
        _ => panic!("Expected Assistant message"),
    }
}
