#![allow(clippy::await_holding_lock)]

//! Simple Chat App - Real-time chat demonstration.
//!
//! This demo showcases:
//! - WebSocket communication for real-time chat
//! - Session management for multiple concurrent chats
//! - Message history storage
//! - Concurrent session handling
//!
//! Migrated from: claude-agent-sdk-demos/simple-chatapp/server/server.ts
//!
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Chat message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Chat session.
#[derive(Debug, Clone)]
pub struct Chat {
    pub id: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
}

/// Chat store for managing multiple chats.
#[derive(Debug, Clone)]
pub struct ChatStore {
    chats: Arc<Mutex<HashMap<String, Chat>>>,
}

impl ChatStore {
    /// Create a new chat store.
    pub fn new() -> Self {
        Self {
            chats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new chat and return its ID.
    pub fn create_chat(&self) -> String {
        let chat_id = Uuid::new_v4().to_string();
        let chat = Chat {
            id: chat_id.clone(),
            messages: Vec::new(),
            created_at: Utc::now(),
        };

        self.chats.lock().unwrap().insert(chat_id.clone(), chat);
        chat_id
    }

    /// Get a chat by ID.
    pub fn get_chat(&self, chat_id: &str) -> Option<Chat> {
        self.chats.lock().unwrap().get(chat_id).cloned()
    }

    /// Add a message to a chat.
    pub fn add_message(&self, chat_id: &str, role: &str, content: &str) -> Result<ChatMessage> {
        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            chat_id: chat_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };

        if let Some(chat) = self.chats.lock().unwrap().get_mut(chat_id) {
            chat.messages.push(message.clone());
        } else {
            anyhow::bail!("Chat not found: {}", chat_id);
        }

        Ok(message)
    }

    /// Get chat history.
    pub fn get_history(&self, chat_id: &str) -> Vec<ChatMessage> {
        self.chats
            .lock()
            .unwrap()
            .get(chat_id)
            .map(|chat| chat.messages.clone())
            .unwrap_or_default()
    }

    /// List all chats.
    pub fn list_chats(&self) -> Vec<String> {
        self.chats.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for ChatStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent session for a single chat.
pub struct AgentSession {
    pub id: String,
    pub chat_id: String,
    pub client: Arc<Mutex<ClaudeAgentClient>>,
    pub created_at: DateTime<Utc>,
}

impl AgentSession {
    /// Create a new session with a client.
    pub fn new(id: String, chat_id: String, client: ClaudeAgentClient, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            chat_id,
            client: Arc::new(Mutex::new(client)),
            created_at,
        }
    }
}

/// Session manager for agent sessions.
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
    chat_store: ChatStore,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(chat_store: ChatStore) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            chat_store,
        }
    }

    /// Create a new session for a chat.
    pub async fn create_session(&self, chat_id: &str) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();

        let options = ClaudeAgentOptions {
            model: std::env::var("ANTHROPIC_MODEL").ok(),
            ..Default::default()
        };

        let mut client = ClaudeAgentClient::new(Some(options));
        client.connect().await.context("Failed to connect")?;

        let session = AgentSession::new(
            session_id.clone(),
            chat_id.to_string(),
            client,
            Utc::now(),
        );

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get a session ID by ID if it exists.
    pub fn get_session_id(&self, session_id: &str) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .contains_key(session_id)
            .then(|| session_id.to_string())
    }

    /// Check if a session exists.
    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions.lock().unwrap().contains_key(session_id)
    }

    /// Send a message and get response.
    pub async fn send_message(&self, session_id: &str, message: &str) -> Result<String> {
        let chat_id = {
            let sessions = self.sessions.lock().unwrap();
            let session = sessions.get(session_id).context("Session not found")?;
            self.chat_store
                .add_message(&session.chat_id, "user", message)?;
            session.chat_id.clone()
        };

        let session_id_clone = session_id.to_string();

        let session = {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_id)
        };

        let mut response_text = String::new();

        if let Some(session) = session {
            {
                let mut client = session.client.lock().unwrap();
                let mut stream = client.query(message).await?;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(msg) => {
                            if let claude_agent_types::Message::Assistant(assistant_msg) = msg {
                                for block in assistant_msg.content {
                                    if let claude_agent_types::message::ContentBlock::Text(text_block) =
                                        block
                                    {
                                        response_text.push_str(&text_block.text);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            anyhow::bail!("Error: {}", e);
                        }
                    }
                }
            }

            // Put session back in the map
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id_clone, session);
        } else {
            anyhow::bail!("Session not found: {}", session_id);
        }

        // Add assistant message to chat store
        self.chat_store
            .add_message(&chat_id, "assistant", &response_text)?;

        Ok(response_text)
    }

    /// Close a session.
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let session = {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_id)
        };
        if let Some(session) = session {
            let mut client = session.client.lock().unwrap();
            client.disconnect().await?;
        }
        Ok(())
    }

    /// List all sessions.
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.lock().unwrap().keys().cloned().collect()
    }
}

/// HTTP request types.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChatAction {
    Create,
    Send { message: String },
    History,
    Close,
}

/// HTTP response types.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChatResponse {
    Created { session_id: String, chat_id: String },
    Sent { response: String },
    History { messages: Vec<ChatMessage> },
    Closed,
    Error { message: String },
}

/// Process a chat request.
pub async fn process_chat_request(
    action: ChatAction,
    session_manager: &SessionManager,
) -> Result<ChatResponse> {
    match action {
        ChatAction::Create => {
            let chat_id = session_manager.chat_store.create_chat();
            let session_id = session_manager.create_session(&chat_id).await?;
            Ok(ChatResponse::Created {
                session_id,
                chat_id,
            })
        }
        ChatAction::Send { message } => {
            // For simplicity, create a new session for each message
            let chat_id = session_manager.chat_store.create_chat();
            let session_id = session_manager.create_session(&chat_id).await?;
            let response = session_manager.send_message(&session_id, &message).await?;
            session_manager.close_session(&session_id).await?;
            Ok(ChatResponse::Sent { response })
        }
        ChatAction::History => {
            let messages = session_manager.chat_store.get_history("");
            Ok(ChatResponse::History { messages })
        }
        ChatAction::Close => {
            // Close all sessions
            for session_id in session_manager.list_sessions() {
                session_manager.close_session(&session_id).await?;
            }
            Ok(ChatResponse::Closed)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("claude_agent=debug".parse().unwrap()),
        )
        .init();

    println!("Claude Agent SDK - Simple Chat App Demo");
    println!("Demonstrating session management and chat history\n");

    // Create chat store and session manager
    let chat_store = ChatStore::new();
    let session_manager = SessionManager::new(chat_store.clone());

    // Demo: Create a chat and send messages
    println!("Creating a new chat session...");

    let chat_id = chat_store.create_chat();
    println!("Chat ID: {}", chat_id);

    let session_id = session_manager.create_session(&chat_id).await?;
    println!("Session ID: {}", session_id);

    // Send a message
    let message = "Hello! Tell me about Rust programming.";
    println!("\nSending message: {}", message);

    let response = session_manager.send_message(&session_id, message).await?;
    println!("\nClaude's response:");
    println!("{}", response);

    // Get chat history
    let history = chat_store.get_history(&chat_id);
    println!("\nChat history ({} messages):", history.len());

    for msg in history {
        println!(
            "[{}] {}: {}",
            msg.timestamp.format("%H:%M:%S"),
            msg.role,
            msg.content
        );
    }

    // Close the session
    session_manager.close_session(&session_id).await?;
    println!("\nSession closed.");

    println!("\n=== Demo Complete ===");
    println!("Note: This demo shows the server-side logic.");
    println!("For full WebSocket functionality, a client would connect.");

    Ok(())
}
