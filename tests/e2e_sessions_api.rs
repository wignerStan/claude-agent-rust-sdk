//! E2E tests for session management.
//!
//! All tests require: claude CLI + API key.
//! Run: `cargo test -p claude-agent-api --test e2e_sessions -- --ignored`

use claude_agent::api::sessions::{get_session_info, get_session_messages, list_sessions};
use claude_agent::api::ClaudeAgentClient;

mod e2e_common;

#[tokio::test]
#[ignore]
async fn test_live_session_listing() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");
    let session_id = client.session_id().expect("session_id should be set").to_string();
    client.disconnect().await.ok();

    // List sessions should include our session
    let sessions: Vec<claude_agent::api::sessions::SessionInfo> =
        list_sessions().await.expect("list_sessions should succeed");
    assert!(
        sessions.iter().any(|s| s.id == session_id),
        "Session {session_id} should appear in listing"
    );
}

#[tokio::test]
#[ignore]
async fn test_live_session_messages_persisted() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");
    let session_id = client.session_id().expect("session_id should be set").to_string();

    // Send a query to generate messages
    {
        let mut stream = client.query("Say hello").await.expect("query failed");
        e2e_common::collect_until_result(&mut stream, e2e_common::STANDARD_TIMEOUT).await;
    } // stream dropped here, releasing borrow on client
    client.disconnect().await.ok();

    // Verify messages were persisted
    let messages = get_session_messages(&session_id).await;
    assert!(messages.is_ok(), "get_session_messages should succeed");
    let msgs = messages.unwrap();
    assert!(!msgs.is_empty(), "Session should have persisted messages");
}

#[tokio::test]
#[ignore]
async fn test_live_session_info_retrieval() {
    e2e_common::init_tracing();
    let mut client = ClaudeAgentClient::new(Some(e2e_common::live_options()));
    client.connect().await.expect("connect failed");
    let session_id = client.session_id().expect("session_id should be set").to_string();
    client.disconnect().await.ok();

    let info = get_session_info(&session_id).await;
    assert!(info.is_ok(), "get_session_info should succeed");
    let info = info.unwrap();
    assert_eq!(info.id, session_id);
}
