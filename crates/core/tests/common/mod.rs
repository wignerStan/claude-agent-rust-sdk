use async_trait::async_trait;
use claude_agent_transport::Transport;
use claude_agent_types::ClaudeAgentError;
use futures::stream::BoxStream;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct MockTransport {
    pub sent_messages: Arc<Mutex<Vec<String>>>,
    tx: broadcast::Sender<Result<serde_json::Value, ClaudeAgentError>>,
}

impl MockTransport {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { sent_messages: Arc::new(Mutex::new(Vec::new())), tx }
    }

    pub async fn push_incoming(&self, json: serde_json::Value) {
        let _ = self.tx.send(Ok(json));
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }

    async fn write(&self, message: &str) -> Result<(), ClaudeAgentError> {
        self.sent_messages.lock().unwrap().push(message.to_string());
        Ok(())
    }

    async fn read_messages(&self) -> BoxStream<'_, Result<serde_json::Value, ClaudeAgentError>> {
        let mut rx = self.tx.subscribe();
        let s = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(val) => yield val,
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(_) => continue,
                }
            }
        };
        Box::pin(s)
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }
}
