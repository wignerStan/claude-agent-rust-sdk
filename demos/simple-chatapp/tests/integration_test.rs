//! Tests for Simple Chat App demo.

use chrono::{DateTime, Utc};
use claude_agent_api::ClaudeAgentClient;
use claude_agent_demos_common::test_utils::{MockTransport, StreamCollector};
use claude_agent_types::message::Message;
use claude_agent_types::ClaudeAgentError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_chat_message_structure() {
    let message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        chat_id: "test-chat-123".to_string(),
        role: "user".to_string(),
        content: "Hello, Claude!".to_string(),
        timestamp: Utc::now(),
    };

    assert!(!message.id.is_empty());
    assert_eq!(message.chat_id, "test-chat-123");
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello, Claude!");
}

#[tokio::test]
async fn test_chat_store_creation() {
    let chat_store = ChatStore::new();
    let chat_id = chat_store.create_chat();

    assert!(!chat_id.is_empty());
    assert!(chat_store.get_chat(&chat_id).is_some());
}

#[tokio::test]
async fn test_chat_message_addition() {
    let chat_store = ChatStore::new();
    let chat_id = chat_store.create_chat();

    let result = chat_store.add_message(&chat_id, "user", "Test message");

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Test message");
}

#[tokio::test]
async fn test_chat_history_retrieval() {
    let chat_store = ChatStore::new();
    let chat_id = chat_store.create_chat();

    // Add some messages
    chat_store
        .add_message(&chat_id, "user", "Message 1")
        .unwrap();
    chat_store
        .add_message(&chat_id, "assistant", "Response 1")
        .unwrap();
    chat_store
        .add_message(&chat_id, "user", "Message 2")
        .unwrap();

    let history = chat_store.get_history(&chat_id);

    assert_eq!(history.len(), 3);
    assert_eq!(history[0].content, "Message 1");
    assert_eq!(history[1].content, "Response 1");
    assert_eq!(history[2].content, "Message 2");
}

#[tokio::test]
async fn test_chat_query_integration() {
    timeout(TEST_TIMEOUT, async {
        let response_text = "Hello! I can help you with Rust programming.";
        let response = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": response_text}],
                "role": "assistant",
                "model": "claude-sonnet-4-5"
            }
        });

        let mock_transport = MockTransport::new(vec![response]);
        let mut client = ClaudeAgentClient::new(None);
        client.set_transport(Box::new(mock_transport));
        client.connect().await.unwrap();

        let stream = client.query("Hello! Tell me about Rust.").await.unwrap();
        let messages = StreamCollector::collect::<Message, ClaudeAgentError>(stream).await.unwrap();

        assert_eq!(messages.len(), 1);
        if let Message::Assistant(msg) = &messages[0] {
            if let claude_agent_types::message::ContentBlock::Text(text_block) = &msg.content[0] {
                assert_eq!(text_block.text, response_text);
            } else {
                panic!("Expected text block");
            }
        } else {
            panic!("Expected assistant message");
        }
    })
    .await
    .expect("Test timed out");
}

#[tokio::test]
async fn test_multiple_chats() {
    let chat_store = ChatStore::new();

    let chat_id_1 = chat_store.create_chat();
    let chat_id_2 = chat_store.create_chat();

    chat_store
        .add_message(&chat_id_1, "user", "Chat 1 message")
        .unwrap();
    chat_store
        .add_message(&chat_id_2, "user", "Chat 2 message")
        .unwrap();

    let history_1 = chat_store.get_history(&chat_id_1);
    let history_2 = chat_store.get_history(&chat_id_2);

    assert_eq!(history_1.len(), 1);
    assert_eq!(history_2.len(), 1);
    assert_eq!(history_1[0].content, "Chat 1 message");
    assert_eq!(history_2[0].content, "Chat 2 message");
}

#[tokio::test]
async fn test_chat_listing() {
    let chat_store = ChatStore::new();

    let chat_id_1 = chat_store.create_chat();
    let chat_id_2 = chat_store.create_chat();
    let chat_id_3 = chat_store.create_chat();

    let chats = chat_store.list_chats();

    assert_eq!(chats.len(), 3);
    assert!(chats.contains(&chat_id_1));
    assert!(chats.contains(&chat_id_2));
    assert!(chats.contains(&chat_id_3));
}

#[tokio::test]
async fn test_session_manager_creation() {
    let chat_store = ChatStore::new();
    let session_manager = SessionManager::new(chat_store);

    let sessions = session_manager.list_sessions();
    assert!(sessions.is_empty());
}

#[tokio::test]
async fn test_chat_action_serialization() {
    let create_action = ChatAction::Create;
    let send_action = ChatAction::Send {
        message: "Hello".to_string(),
    };
    let history_action = ChatAction::History;
    let close_action = ChatAction::Close;

    // Test JSON serialization
    let create_json = serde_json::to_string(&create_action).unwrap();
    let send_json = serde_json::to_string(&send_action).unwrap();
    let history_json = serde_json::to_string(&history_action).unwrap();
    let close_json = serde_json::to_string(&close_action).unwrap();

    assert!(create_json.contains("Create"));
    assert!(send_json.contains("Hello"));
    assert!(history_json.contains("History"));
    assert!(close_json.contains("Close"));
}

#[tokio::test]
async fn test_chat_response_serialization() {
    let created_response = ChatResponse::Created {
        session_id: "session-123".to_string(),
        chat_id: "chat-456".to_string(),
    };
    let sent_response = ChatResponse::Sent {
        response: "Hello!".to_string(),
    };
    let closed_response = ChatResponse::Closed;

    let created_json = serde_json::to_string(&created_response).unwrap();
    let sent_json = serde_json::to_string(&sent_response).unwrap();
    let closed_json = serde_json::to_string(&closed_response).unwrap();

    assert!(created_json.contains("session-123"));
    assert!(sent_json.contains("Hello!"));
    assert!(closed_json.contains("Closed"));
}

#[tokio::test]
async fn test_message_serialization_roundtrip() {
    let original_message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        chat_id: "test-chat".to_string(),
        role: "assistant".to_string(),
        content: "I can help with that!".to_string(),
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&original_message).unwrap();
    let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();

    assert_eq!(original_message.id, deserialized.id);
    assert_eq!(original_message.chat_id, deserialized.chat_id);
    assert_eq!(original_message.role, deserialized.role);
    assert_eq!(original_message.content, deserialized.content);
}

// Simplified structures for testing (without actual SDK dependencies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Chat {
    pub id: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ChatStore {
    chats: Arc<Mutex<HashMap<String, Chat>>>,
}

impl ChatStore {
    pub fn new() -> Self {
        Self {
            chats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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

    pub fn get_chat(&self, chat_id: &str) -> Option<Chat> {
        self.chats.lock().unwrap().get(chat_id).cloned()
    }

    pub fn add_message(
        &self,
        chat_id: &str,
        role: &str,
        content: &str,
    ) -> Result<ChatMessage, String> {
        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            chat_id: chat_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };

        if let Some(chat) = self.chats.lock().unwrap().get_mut(chat_id) {
            chat.messages.push(message.clone());
            Ok(message)
        } else {
            Err("Chat not found".to_string())
        }
    }

    pub fn get_history(&self, chat_id: &str) -> Vec<ChatMessage> {
        self.chats
            .lock()
            .unwrap()
            .get(chat_id)
            .map(|chat| chat.messages.clone())
            .unwrap_or_default()
    }

    pub fn list_chats(&self) -> Vec<String> {
        self.chats.lock().unwrap().keys().cloned().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub chat_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
    pub chat_store: ChatStore,
}

impl SessionManager {
    pub fn new(chat_store: ChatStore) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            chat_store,
        }
    }

    pub fn create_session(&self, chat_id: &str) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = AgentSession {
            id: session_id.clone(),
            chat_id: chat_id.to_string(),
            created_at: Utc::now(),
        };
        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);
        session_id
    }

    pub fn get_session(&self, session_id: &str) -> Option<AgentSession> {
        self.sessions.lock().unwrap().get(session_id).cloned()
    }

    pub fn close_session(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
    }

    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.lock().unwrap().keys().cloned().collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChatAction {
    Create,
    Send { message: String },
    History,
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChatResponse {
    Created { session_id: String, chat_id: String },
    Sent { response: String },
    History { messages: Vec<ChatMessage> },
    Closed,
    Error { message: String },
}
