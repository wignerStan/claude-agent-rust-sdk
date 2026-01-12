//! Message streaming utilities.

use futures::stream::BoxStream;
use tokio::sync::mpsc;

use claude_agent_types::{ClaudeAgentError, Message};

/// Create a message channel for streaming.
pub fn message_channel(buffer_size: usize) -> (MessageSender, MessageReceiver) {
    let (tx, rx) = mpsc::channel(buffer_size);
    (MessageSender { tx }, MessageReceiver { rx })
}

/// Sender side of a message channel.
pub struct MessageSender {
    tx: mpsc::Sender<Result<Message, ClaudeAgentError>>,
}

impl MessageSender {
    /// Send a message.
    pub async fn send(&self, message: Message) -> Result<(), ClaudeAgentError> {
        self.tx
            .send(Ok(message))
            .await
            .map_err(|e| ClaudeAgentError::Transport(format!("Failed to send message: {}", e)))
    }

    /// Send an error.
    pub async fn send_error(&self, error: ClaudeAgentError) -> Result<(), ClaudeAgentError> {
        self.tx
            .send(Err(error))
            .await
            .map_err(|e| ClaudeAgentError::Transport(format!("Failed to send error: {}", e)))
    }

    /// Check if the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

/// Receiver side of a message channel.
pub struct MessageReceiver {
    rx: mpsc::Receiver<Result<Message, ClaudeAgentError>>,
}

impl MessageReceiver {
    /// Receive a message.
    pub async fn recv(&mut self) -> Option<Result<Message, ClaudeAgentError>> {
        self.rx.recv().await
    }

    /// Convert to a boxed stream.
    pub fn into_stream(self) -> BoxStream<'static, Result<Message, ClaudeAgentError>> {
        Box::pin(futures::stream::unfold(self.rx, |mut rx| async move {
            rx.recv().await.map(|msg| (msg, rx))
        }))
    }
}
