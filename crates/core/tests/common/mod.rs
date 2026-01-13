use async_trait::async_trait;
use claude_agent_transport::Transport;
use claude_agent_types::ClaudeAgentError;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct MockTransport {
    pub sent_messages: Arc<Mutex<Vec<String>>>,
    tx: mpsc::Sender<Result<serde_json::Value, ClaudeAgentError>>,
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Result<serde_json::Value, ClaudeAgentError>>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            tx,
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }

    pub async fn push_incoming(&self, json: serde_json::Value) {
        let _ = self.tx.send(Ok(json)).await;
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
        let rx = self.rx.clone();
        stream::unfold(rx, |rx| async move {
            let mut guard = rx.lock().await;
            let msg = guard.recv().await;
            drop(guard);
            msg.map(|m| (m, rx))
        })
        .boxed()
    }

    async fn close(&mut self) -> Result<(), ClaudeAgentError> {
        Ok(())
    }
}
