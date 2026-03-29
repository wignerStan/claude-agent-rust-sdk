//! Comprehensive integration tests for ClaudeAgentClient.
//!
//! These tests cover message parsing, streaming events, re-exports,
//! and edge cases not covered by the existing integration tests.

use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::json;
use std::sync::{Arc, Mutex};

use claude_agent_api::{query, ClaudeAgentClient};
use claude_agent_transport::Transport;
use claude_agent_types::message::{ContentBlock, Message};
use claude_agent_types::{ClaudeAgentError, ClaudeAgentOptions};

// --- Mock Transport ---

struct MockTransport {
    sent_data: Arc<Mutex<Vec<String>>>,
    responses: Vec<serde_json::Value>,
}

impl MockTransport {
    fn new(responses: Vec<serde_json::Value>) -> Self {
        Self { sent_data: Arc::new(Mutex::new(Vec::new())), responses }
    }

    fn sent_data_clone(&self) -> Arc<Mutex<Vec<String>>> {
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

// --- Helpers ---

async fn connected_client_async(
    responses: Vec<serde_json::Value>,
) -> (ClaudeAgentClient, Arc<Mutex<Vec<String>>>) {
    let mock = MockTransport::new(responses);
    let sent_data = mock.sent_data_clone();
    let mut client = ClaudeAgentClient::new(Some(ClaudeAgentOptions::default()));
    client.set_transport(Box::new(mock));
    client.connect().await.unwrap();
    (client, sent_data)
}

async fn collect_messages(
    stream: BoxStream<'_, Result<Message, ClaudeAgentError>>,
) -> Vec<Message> {
    let mut messages = Vec::new();
    futures::pin_mut!(stream);
    while let Some(result) = stream.next().await {
        messages.push(result.expect("Stream item error"));
    }
    messages
}

// --- Construction tests ---

#[tokio::test]
async fn client_with_default_options() {
    let client = ClaudeAgentClient::new(None);
    assert!(client.session_id().is_none());
}

#[tokio::test]
async fn client_with_explicit_options() {
    let opts = ClaudeAgentOptions::default();
    let client = ClaudeAgentClient::new(Some(opts));
    assert!(client.session_id().is_none());
}

// --- Connect / disconnect ---

#[tokio::test]
async fn connect_and_disconnect() {
    let (mut client, _) = connected_client_async(vec![]).await;
    assert!(client.session_id().is_some());
    assert!(client.disconnect().await.is_ok());
}

#[tokio::test]
async fn connect_sets_session_id() {
    let (client, _) = connected_client_async(vec![]).await;
    let session = client.session_id();
    assert!(session.is_some());
    assert!(!session.unwrap().is_empty());
}

// --- Message type parsing ---

#[tokio::test]
async fn assistant_text_message() {
    let response = json!({
        "type": "assistant",
        "message": {
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-test"
        }
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 1);
            match &msg.content[0] {
                ContentBlock::Text(t) => assert_eq!(t.text, "Hello!"),
                other => panic!("Expected Text block, got: {:?}", other),
            }
        },
        other => panic!("Expected Assistant message, got: {:?}", other),
    }
}

#[tokio::test]
async fn result_message() {
    let response = json!({
        "type": "result",
        "subtype": "success",
        "duration_ms": 500,
        "duration_api_ms": 400,
        "is_error": false,
        "num_turns": 1,
        "session_id": "sess-123",
        "total_cost_usd": 0.001
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Result(r) => {
            assert_eq!(r.session_id, "sess-123");
            assert_eq!(r.duration_ms, 500);
            assert!(!r.is_error);
            assert_eq!(r.total_cost_usd, Some(0.001));
        },
        other => panic!("Expected Result message, got: {:?}", other),
    }
}

#[tokio::test]
async fn system_message() {
    // System init messages are filtered out by the agent's query stream
    // (handled by the background task). Use a non-init subtype to test parsing.
    let response = json!({
        "type": "system",
        "subtype": "mcp_connected",
        "data": {"server": "test-mcp"}
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::System(s) => {
            assert_eq!(s.subtype, "mcp_connected");
            assert_eq!(s.data["server"], "test-mcp");
        },
        other => panic!("Expected System message, got: {:?}", other),
    }
}

#[tokio::test]
async fn error_message() {
    let response = json!({
        "type": "error",
        "error": {
            "type": "api_error",
            "message": "Rate limited"
        }
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Error(e) => {
            assert_eq!(e.error.error_type, "api_error");
            assert_eq!(e.error.message, "Rate limited");
        },
        other => panic!("Expected Error message, got: {:?}", other),
    }
}

// --- Multi-message streaming ---

#[tokio::test]
async fn multiple_messages_in_sequence() {
    let responses = vec![
        json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "Part 1"}], "model": "test"}}),
        json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "Part 2"}], "model": "test"}}),
        json!({"type": "result", "subtype": "success", "duration_ms": 100, "duration_api_ms": 80, "is_error": false, "num_turns": 1, "session_id": "s1"}),
    ];
    let (mut client, _) = connected_client_async(responses).await;
    let stream = client.query("multi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 3);
    assert!(matches!(&messages[0], Message::Assistant(_)));
    assert!(matches!(&messages[1], Message::Assistant(_)));
    assert!(matches!(&messages[2], Message::Result(_)));
}

// --- Thinking blocks ---

#[tokio::test]
async fn thinking_block_in_assistant_message() {
    let response = json!({
        "type": "assistant",
        "message": {
            "role": "assistant",
            "content": [
                {"type": "thinking", "thinking": "Let me think...", "signature": "sig123"},
                {"type": "text", "text": "The answer is 42."}
            ],
            "model": "test"
        }
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("think").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 2);
            match &msg.content[0] {
                ContentBlock::Thinking(t) => {
                    assert_eq!(t.thinking, "Let me think...");
                    assert_eq!(t.signature, "sig123");
                },
                other => panic!("Expected Thinking block, got: {:?}", other),
            }
            match &msg.content[1] {
                ContentBlock::Text(t) => assert_eq!(t.text, "The answer is 42."),
                other => panic!("Expected Text block, got: {:?}", other),
            }
        },
        other => panic!("Expected Assistant message, got: {:?}", other),
    }
}

// --- Tool use blocks ---

#[tokio::test]
async fn tool_use_block_in_assistant_message() {
    let response = json!({
        "type": "assistant",
        "message": {
            "role": "assistant",
            "content": [
                {"type": "tool_use", "id": "tool-1", "name": "Read", "input": {"file_path": "/tmp/test.rs"}}
            ],
            "model": "test"
        }
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("read").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 1);
            match &msg.content[0] {
                ContentBlock::ToolUse(t) => {
                    assert_eq!(t.id, "tool-1");
                    assert_eq!(t.name, "Read");
                    assert_eq!(t.input["file_path"], "/tmp/test.rs");
                },
                other => panic!("Expected ToolUse block, got: {:?}", other),
            }
        },
        other => panic!("Expected Assistant message, got: {:?}", other),
    }
}

// --- Transport write verification ---

#[tokio::test]
async fn query_sends_user_message_to_transport() {
    let response = json!({
        "type": "assistant",
        "message": {"role": "assistant", "content": [{"type": "text", "text": "ok"}], "model": "test"}
    });
    let (mut client, sent_data) = connected_client_async(vec![response]).await;
    let mut stream = client.query("Send this").await.unwrap();
    while stream.next().await.is_some() {}

    let sent = sent_data.lock().unwrap();
    assert!(!sent.is_empty());
    let last = sent.last().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(last).unwrap();
    assert_eq!(parsed["type"], "user");
    assert_eq!(parsed["message"]["content"][0]["text"], "Send this");
}

// --- Streaming event types ---

#[tokio::test]
async fn message_start_event() {
    let response = json!({
        "type": "message_start",
        "message": {
            "role": "assistant",
            "content": [],
            "model": "test"
        }
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0], Message::MessageStart(_)));
}

#[tokio::test]
async fn content_block_start_event() {
    let response = json!({
        "type": "content_block_start",
        "index": 0,
        "content_block": {"type": "text", "text": ""}
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0], Message::ContentBlockStart(_)));
}

#[tokio::test]
async fn content_block_delta_event() {
    let response = json!({
        "type": "content_block_delta",
        "index": 0,
        "delta": {"type": "text_delta", "text": "chunk"}
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0], Message::ContentBlockDelta(_)));
}

#[tokio::test]
async fn content_block_stop_event() {
    let response = json!({
        "type": "content_block_stop",
        "index": 0
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0], Message::ContentBlockStop(_)));
}

#[tokio::test]
async fn ping_event() {
    let response = json!({"type": "ping"});
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("hi").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    assert!(matches!(&messages[0], Message::Ping(_)));
}

// --- Result edge cases ---

#[tokio::test]
async fn result_message_with_error_flag() {
    let response = json!({
        "type": "result",
        "subtype": "error",
        "duration_ms": 200,
        "duration_api_ms": 150,
        "is_error": true,
        "num_turns": 3,
        "session_id": "err-sess",
        "result": "Something went wrong"
    });
    let (mut client, _) = connected_client_async(vec![response]).await;
    let stream = client.query("fail").await.unwrap();
    let messages = collect_messages(stream).await;

    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Result(r) => {
            assert!(r.is_error);
            assert_eq!(r.result.as_deref(), Some("Something went wrong"));
            assert_eq!(r.num_turns, 3);
        },
        other => panic!("Expected Result message, got: {:?}", other),
    }
}

#[tokio::test]
async fn empty_responses_yield_no_messages() {
    let (mut client, _) = connected_client_async(vec![]).await;
    let stream = client.query("empty").await.unwrap();
    let messages = collect_messages(stream).await;
    assert!(messages.is_empty());
}

// --- Re-export tests ---

#[test]
fn re_exports_are_accessible() {
    // Verify that the public API re-exports work
    use claude_agent_api::{ClaudeAgentError, ClaudeAgentOptions, Message};
    let _opts: ClaudeAgentOptions = ClaudeAgentOptions::default();
    let _err: ClaudeAgentError = ClaudeAgentError::Transport("test".into());
    // Message is an enum, just verify it's in scope
    let _ = std::mem::size_of::<Message>();
}

#[test]
fn types_module_is_accessible() {
    use claude_agent_api::types;
    let _ = std::mem::size_of::<types::ClaudeAgentError>();
    let _ = std::mem::size_of::<types::ClaudeAgentOptions>();
}

// --- query function tests ---

#[tokio::test]
#[ignore] // Requires interactive CLI session
async fn query_function_fails_without_cli() {
    let result = query("test", None).await;
    assert!(result.is_err());
}

// --- Session info module ---

#[test]
fn session_info_and_find_claude_cli_accessible() {
    use claude_agent_api::{find_claude_cli, SessionInfo};
    // Just verify they compile
    let _ = std::mem::size_of::<SessionInfo>();
    let _result = find_claude_cli();
}
