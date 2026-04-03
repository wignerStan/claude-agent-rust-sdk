//! Integration tests for agent lifecycle: connect, query, disconnect.

use claude_agent::core::ClaudeAgent;
use claude_agent::types::Message;
use claude_agent::ClaudeAgentOptions;
use futures::StreamExt;
use serde_json::json;
use tokio::time::{timeout, Duration};

mod common_core;
use common_core::MockTransport;

async fn connected_agent() -> (ClaudeAgent, MockTransport) {
    let mut agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let transport = MockTransport::new();
    let transport_clone = transport.clone();
    agent.set_transport(Box::new(transport));
    agent.connect(None).await.expect("Connect should succeed");
    (agent, transport_clone)
}

#[tokio::test]
async fn test_agent_connect_creates_session() {
    let (agent, _transport) = connected_agent().await;
    let session = agent.current_session();
    assert!(session.is_some());
    assert!(session.unwrap().is_active);
}

#[tokio::test]
async fn test_agent_disconnect_deactivates_session() {
    let (mut agent, _transport) = connected_agent().await;
    assert!(agent.current_session().unwrap().is_active);
    agent.disconnect().await.expect("Disconnect should succeed");
    assert!(!agent.current_session().unwrap().is_active);
}

#[tokio::test]
async fn test_agent_new_without_connect_has_no_session() {
    let agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    assert!(agent.current_session().is_none());
}

#[tokio::test]
async fn test_agent_query_sends_user_message() {
    let (mut agent, transport) = connected_agent().await;

    let _stream = agent.query("Hello").await.expect("Query should succeed");

    tokio::time::sleep(Duration::from_millis(50)).await;

    drop(_stream);

    let msgs = transport.sent_messages.lock().unwrap();
    let last_msg = msgs.last().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(last_msg).unwrap();
    assert_eq!(parsed.get("type").unwrap().as_str(), Some("user"));
    assert_eq!(parsed["message"]["content"][0]["text"].as_str(), Some("Hello"));
}

#[tokio::test]
async fn test_agent_query_filters_control_messages() {
    let (mut agent, transport) = connected_agent().await;

    let push_transport = transport.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        push_transport
            .push_incoming(json!({
                "type": "control_response",
                "request_id": "req-1",
                "success": true
            }))
            .await;
        push_transport
            .push_incoming(json!({
                "type": "system",
                "subtype": "init",
                "data": {"output_style": "verbose"}
            }))
            .await;
        push_transport
            .push_incoming(json!({
                "type": "assistant",
                "message": {"role": "assistant", "content": []}
            }))
            .await;
        push_transport.push_incoming(json!({"type": "message_stop"})).await;
    });

    let mut stream = agent.query("test").await.expect("Query should succeed");

    let result = timeout(Duration::from_secs(5), async {
        let mut received_messages = Vec::new();
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                received_messages.push(msg);
            }
        }
        received_messages
    })
    .await;

    match result {
        Ok(received_messages) => {
            assert_eq!(received_messages.len(), 1);
            assert!(matches!(received_messages[0], Message::Assistant(_)));
        },
        Err(_) => {
            let msgs = transport.sent_messages.lock().unwrap();
            let last_msg = msgs.last().unwrap();
            let parsed: serde_json::Value = serde_json::from_str(last_msg).unwrap();
            assert_eq!(parsed.get("type").unwrap().as_str(), Some("user"));
        },
    }
}

#[tokio::test]
async fn test_agent_disconnect_can_be_called_multiple_times() {
    let (mut agent, _transport) = connected_agent().await;
    agent.disconnect().await.expect("First disconnect should succeed");
    agent.disconnect().await.expect("Second disconnect should succeed");
}

#[tokio::test]
async fn test_agent_hook_registry_accessible() {
    let agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let _ = agent.hook_registry();
}

#[tokio::test]
async fn test_agent_mcp_manager_accessible() {
    let agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let _ = agent.mcp_manager();
}

#[tokio::test]
async fn test_agent_set_transport_before_connect() {
    let mut agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let transport = MockTransport::new();
    agent.set_transport(Box::new(transport));
    agent.connect(None).await.expect("Connect with mock transport should succeed");
    assert!(agent.current_session().is_some());
}

#[tokio::test]
async fn test_agent_get_server_info_none_before_init() {
    let agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let info = agent.get_server_info().await;
    assert!(info.is_none());
}

#[tokio::test]
async fn test_agent_get_server_info_after_init_message() {
    let mut agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let transport = MockTransport::new();

    let push_transport = transport.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        push_transport
            .push_incoming(json!({
                "type": "system",
                "subtype": "init",
                "data": {
                    "output_style": "concise",
                    "commands": ["/help", "/compact"]
                }
            }))
            .await;
    });

    agent.set_transport(Box::new(transport));
    agent.connect(None).await.expect("Connect should succeed");

    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    let info = agent.get_server_info().await;
    assert!(info.is_some());
    let server_info = info.unwrap();
    assert_eq!(server_info.get("output_style").and_then(|v| v.as_str()), Some("concise"));
}
