//! Session management for Claude Agent SDK.

use std::collections::HashMap;

use uuid::Uuid;

/// Session state for a conversation.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub is_active: bool,
    pub checkpoints: Vec<SessionCheckpoint>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A checkpoint in a session for file rewinding.
#[derive(Debug, Clone)]
pub struct SessionCheckpoint {
    pub user_message_id: String,
    pub timestamp: u64,
}

impl Session {
    /// Create a new session.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            is_active: true,
            checkpoints: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a session with a specific ID.
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            is_active: true,
            checkpoints: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a checkpoint.
    pub fn add_checkpoint(&mut self, user_message_id: String) {
        let checkpoint = SessionCheckpoint {
            user_message_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.checkpoints.push(checkpoint);
    }

    /// Get the last checkpoint.
    pub fn last_checkpoint(&self) -> Option<&SessionCheckpoint> {
        self.checkpoints.last()
    }

    /// Deactivate the session.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Fork the session with a new ID.
    pub fn fork(&self) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            is_active: true,
            checkpoints: self.checkpoints.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager for multiple sessions.
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    current_session_id: Option<String>,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            current_session_id: None,
        }
    }

    /// Create a new session and set it as current.
    pub fn create_session(&mut self) -> &Session {
        let session = Session::new();
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.current_session_id = Some(id.clone());
        self.sessions
            .get(&id)
            .expect("Session should exist after insert")
    }

    /// Get the current session.
    pub fn current_session(&self) -> Option<&Session> {
        self.current_session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id))
    }

    /// Get a mutable reference to the current session.
    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        let id = self.current_session_id.clone()?;
        self.sessions.get_mut(&id)
    }

    /// Get a session by ID.
    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    /// Resume a session by ID.
    pub fn resume_session(&mut self, id: &str) -> Option<&Session> {
        if self.sessions.contains_key(id) {
            self.current_session_id = Some(id.to_string());
            self.sessions.get(id)
        } else {
            None
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
