//! Tests for message streaming: MessageSender, MessageReceiver, message_channel.

use claude_agent_core::streaming::message_channel;
use claude_agent_types::{ClaudeAgentError, Message};
use futures::StreamExt;

fn make_message() -> Message {
    serde_json::from_value(serde_json::json!({"type": "message_stop"})).expect("valid message")
}

#[tokio::test]
async fn send_and_recv_message() {
    let (sender, mut receiver) = message_channel(10);
    sender.send(make_message()).await.unwrap();
    let received = receiver.recv().await.unwrap().unwrap();
    assert!(matches!(received, Message::MessageStop(_)));
}

#[tokio::test]
async fn send_error_and_recv() {
    let (sender, mut receiver) = message_channel(10);
    sender.send_error(ClaudeAgentError::Transport("fail".to_string())).await.unwrap();
    let result = receiver.recv().await.unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("fail"));
}

#[tokio::test]
async fn is_closed_false_when_active() {
    let (sender, _receiver) = message_channel(10);
    assert!(!sender.is_closed());
}

#[tokio::test]
async fn is_closed_after_receiver_dropped() {
    let (sender, receiver) = message_channel(10);
    drop(receiver);
    tokio::task::yield_now().await;
    assert!(sender.is_closed());
}

#[tokio::test]
async fn recv_none_after_sender_dropped() {
    let (sender, mut receiver) = message_channel(10);
    drop(sender);
    assert!(receiver.recv().await.is_none());
}

#[tokio::test]
async fn multiple_messages_in_order() {
    let (sender, mut receiver) = message_channel(100);
    for _i in 0..5 {
        sender.send(make_message()).await.unwrap();
    }
    for _ in 0..5 {
        assert!(receiver.recv().await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn into_stream_yields_messages() {
    let (sender, receiver) = message_channel(10);
    sender.send(make_message()).await.unwrap();
    drop(sender);
    let mut stream = receiver.into_stream();
    assert!(stream.next().await.unwrap().is_ok());
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn into_stream_yields_error() {
    let (sender, receiver) = message_channel(10);
    sender.send_error(ClaudeAgentError::MessageParse("bad".to_string())).await.unwrap();
    drop(sender);
    let mut stream = receiver.into_stream();
    assert!(stream.next().await.unwrap().is_err());
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn send_fails_after_receiver_dropped() {
    let (sender, receiver) = message_channel(1);
    drop(receiver);
    let result = sender.send(make_message()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to send message"));
}

#[tokio::test]
async fn send_error_fails_after_receiver_dropped() {
    let (sender, receiver) = message_channel(1);
    drop(receiver);
    let result = sender.send_error(ClaudeAgentError::Transport("err".to_string())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn small_buffer_channel() {
    let (sender, mut receiver) = message_channel(1);
    tokio::spawn(async move {
        sender.send(make_message()).await.unwrap();
    });
    assert!(receiver.recv().await.unwrap().is_ok());
}
