//! Integration tests for agent control methods using MockTransport.

use claude_agent_core::{ClaudeAgent, ClaudeAgentOptions};
use serde_json::json;

mod common;
use common::MockTransport;

async fn connected_agent() -> (ClaudeAgent, MockTransport) {
    let mut agent = ClaudeAgent::new(ClaudeAgentOptions::default());
    let transport = MockTransport::new();
    let transport_clone = transport.clone();
    agent.set_transport(Box::new(transport));
    agent.connect(None).await.expect("Connect should succeed");
    (agent, transport_clone)
}

fn spawn_responder(transport: MockTransport) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let request_id = {
            let msgs = transport.sent_messages.lock().unwrap();
            let last_msg = msgs.last().unwrap().clone();
            let parsed: serde_json::Value = serde_json::from_str(&last_msg).unwrap();
            parsed.get("request_id").and_then(|v| v.as_str()).unwrap_or("").to_string()
        };
        transport
            .push_incoming(json!({
                "type": "control_response",
                "request_id": request_id,
                "success": true
            }))
            .await;
    })
}

#[tokio::test]
async fn test_agent_interrupt() {
    let (agent, transport) = connected_agent().await;
    let handle = spawn_responder(transport.clone());
    let _ = agent.interrupt().await;
    handle.await.unwrap();
    let msgs = transport.sent_messages.lock().unwrap();
    let last_msg = msgs.last().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(last_msg).unwrap();
    assert_eq!(parsed.get("type").unwrap().as_str(), Some("control_request"));
    assert_eq!(
        parsed.get("request").unwrap().get("subtype").unwrap().as_str(),
        Some("interrupt")
    );
}

#[tokio::test]
async fn test_agent_set_permission_mode() {
    let (agent, transport) = connected_agent().await;
    let handle = spawn_responder(transport.clone());
    let _ = agent.set_permission_mode("plan").await;
    handle.await.unwrap();
    let msgs = transport.sent_messages.lock().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(msgs.last().unwrap()).unwrap();
    let request = parsed.get("request").unwrap();
    assert_eq!(request.get("subtype").unwrap().as_str(), Some("set_permission_mode"));
    assert_eq!(request.get("mode").unwrap().as_str(), Some("plan"));
}

#[tokio::test]
async fn test_agent_set_model_some() {
    let (agent, transport) = connected_agent().await;
    let handle = spawn_responder(transport.clone());
    let _ = agent.set_model(Some("claude-opus-4")).await;
    handle.await.unwrap();
    let msgs = transport.sent_messages.lock().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(msgs.last().unwrap()).unwrap();
    assert_eq!(
        parsed.get("request").unwrap().get("model").unwrap().as_str(),
        Some("claude-opus-4")
    );
}

#[tokio::test]
async fn test_agent_set_model_none() {
    let (agent, transport) = connected_agent().await;
    let handle = spawn_responder(transport.clone());
    let _ = agent.set_model(None).await;
    handle.await.unwrap();
    let msgs = transport.sent_messages.lock().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(msgs.last().unwrap()).unwrap();
    assert_eq!(parsed.get("request").unwrap().get("model").unwrap(), &json!(null));
}

#[tokio::test]
async fn test_agent_rewind_files() {
    let (agent, transport) = connected_agent().await;
    let handle = spawn_responder(transport.clone());
    let _ = agent.rewind_files("msg-uuid-42").await;
    handle.await.unwrap();
    let msgs = transport.sent_messages.lock().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(msgs.last().unwrap()).unwrap();
    assert_eq!(
        parsed.get("request").unwrap().get("subtype").unwrap().as_str(),
        Some("rewind_files")
    );
}
