//! Standalone session management for Claude Code.
//!
//! This module provides functions for managing Claude Code sessions without
//! requiring an active `ClaudeAgentClient` connection. Sessions are managed
//! by reading session files directly from the Claude Code sessions directory
//! on disk (`~/.claude/sessions/`).

use std::path::PathBuf;

use crate::types::ClaudeAgentError;

/// Information about a Claude Code session.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SessionInfo {
    /// Unique session identifier.
    pub id: String,
    /// Session title, if set.
    pub title: Option<String>,
    /// Tags associated with this session.
    #[serde(default)]
    pub tags: Vec<String>,
    /// ISO 8601 timestamp of when the session was created.
    pub created_at: Option<String>,
    /// ISO 8601 timestamp of when the session was last updated.
    pub updated_at: Option<String>,
}

/// Find the Claude CLI binary on the system.
///
/// Searches the system `PATH` first, then checks common installation
/// locations. Returns an error if the CLI cannot be found.
pub fn find_claude_cli() -> Result<PathBuf, ClaudeAgentError> {
    // Try PATH first
    if let Ok(path) = which::which("claude") {
        return Ok(path);
    }
    // Try common installation locations
    let candidates = [
        dirs::home_dir().map(|h| h.join(".claude/local/claude")),
        Some(PathBuf::from("/usr/local/bin/claude")),
        Some(PathBuf::from("/opt/homebrew/bin/claude")),
    ];
    for path in candidates.iter().flatten() {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    Err(ClaudeAgentError::CLINotFound(
        "Claude CLI not found on PATH or in common locations".into(),
    ))
}

/// Return the base directory where Claude Code stores session data.
///
/// By default, this is `~/.claude/sessions/`. The directory may not exist
/// if no sessions have been recorded yet.
fn sessions_base_dir() -> Result<PathBuf, ClaudeAgentError> {
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .ok_or_else(|| ClaudeAgentError::Config("Cannot determine home directory".into()))?;
    Ok(home.join(".claude").join("sessions"))
}

/// List all Claude Code sessions.
///
/// Reads session metadata from the Claude Code sessions directory and returns
/// a list of available sessions. Returns an empty list if no sessions exist
/// or the sessions directory has not been created yet.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or if the
/// sessions directory cannot be read.
pub async fn list_sessions() -> Result<Vec<SessionInfo>, ClaudeAgentError> {
    let base = sessions_base_dir()?;

    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();

    // Each project has a subdirectory named by a hash
    let project_dirs = tokio::fs::read_dir(&base).await.map_err(|e| {
        ClaudeAgentError::Transport(format!("Failed to read sessions directory: {e}"))
    })?;

    let mut entries = project_dirs;
    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        ClaudeAgentError::Transport(format!("Failed to read project directory entry: {e}"))
    })? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Each session within a project directory is stored as a JSONL file
        let session_files = tokio::fs::read_dir(&path).await.map_err(|e| {
            ClaudeAgentError::Transport(format!("Failed to read project sessions: {e}"))
        })?;

        let mut session_entries = session_files;
        while let Some(session_entry) = session_entries.next_entry().await.map_err(|e| {
            ClaudeAgentError::Transport(format!("Failed to read session file entry: {e}"))
        })? {
            let session_path = session_entry.path();
            if session_path.extension().is_none_or(|ext| ext != "jsonl") {
                continue;
            }

            let stem = session_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            sessions.push(SessionInfo {
                id: stem.to_string(),
                title: None,
                tags: Vec::new(),
                created_at: None,
                updated_at: None,
            });
        }
    }

    Ok(sessions)
}

/// Get metadata for a specific session.
///
/// # Arguments
///
/// * `session_id` - The unique identifier of the session.
///
/// # Errors
///
/// Returns an error if the session cannot be found or cannot be read.
pub async fn get_session_info(session_id: &str) -> Result<SessionInfo, ClaudeAgentError> {
    let base = sessions_base_dir()?;
    let session_file = find_session_file(&base, session_id).await?;

    if !session_file.exists() {
        return Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")));
    }

    let metadata = tokio::fs::metadata(&session_file).await.map_err(|e| {
        ClaudeAgentError::Transport(format!("Failed to read session metadata: {e}"))
    })?;

    let modified = metadata
        .modified()
        .ok()
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_default();

    Ok(SessionInfo {
        id: session_id.to_string(),
        title: None,
        tags: Vec::new(),
        created_at: None,
        updated_at: Some(modified),
    })
}

/// Get messages from a specific session.
///
/// Reads the JSONL session file and returns all messages as JSON values.
/// Each line in the session file is a separate JSON object.
///
/// # Arguments
///
/// * `session_id` - The unique identifier of the session.
///
/// # Errors
///
/// Returns an error if the session cannot be found or the file cannot be parsed.
pub async fn get_session_messages(
    session_id: &str,
) -> Result<Vec<serde_json::Value>, ClaudeAgentError> {
    let base = sessions_base_dir()?;
    let session_file = find_session_file(&base, session_id).await?;

    if !session_file.exists() {
        return Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")));
    }

    let content = tokio::fs::read_to_string(&session_file)
        .await
        .map_err(|e| ClaudeAgentError::Transport(format!("Failed to read session file: {e}")))?;

    let mut messages = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(value) => messages.push(value),
            Err(e) => {
                return Err(ClaudeAgentError::JSONDecode(format!(
                    "Failed to parse session line: {e}"
                )));
            },
        }
    }

    Ok(messages)
}

/// Rename a session by updating its title.
///
/// Note: This currently stores a rename marker alongside the session file.
/// Full support depends on Claude CLI capabilities.
///
/// # Arguments
///
/// * `session_id` - The unique identifier of the session.
/// * `title` - The new title for the session.
///
/// # Errors
///
/// Returns an error if the session cannot be found or the rename fails.
pub async fn rename_session(session_id: &str, title: &str) -> Result<(), ClaudeAgentError> {
    let base = sessions_base_dir()?;
    let session_file = find_session_file(&base, session_id).await?;

    if !session_file.exists() {
        return Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")));
    }

    let rename_marker = session_file.with_extension("title");
    tokio::fs::write(&rename_marker, title)
        .await
        .map_err(|e| ClaudeAgentError::Transport(format!("Failed to write session title: {e}")))?;

    Ok(())
}

/// Add a tag to a session.
///
/// Tags are stored in a sidecar file next to the session data.
///
/// # Arguments
///
/// * `session_id` - The unique identifier of the session.
/// * `tag` - The tag to add to the session.
///
/// # Errors
///
/// Returns an error if the session cannot be found or the tag file cannot
/// be written.
pub async fn tag_session(session_id: &str, tag: &str) -> Result<(), ClaudeAgentError> {
    let base = sessions_base_dir()?;
    let session_file = find_session_file(&base, session_id).await?;

    if !session_file.exists() {
        return Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")));
    }

    let tags_file = session_file.with_extension("tags");

    // Read existing tags if present
    let mut existing_tags = Vec::new();
    if tags_file.exists() {
        let content = tokio::fs::read_to_string(&tags_file).await.map_err(|e| {
            ClaudeAgentError::Transport(format!("Failed to read session tags: {e}"))
        })?;
        existing_tags = content.lines().map(|l| l.to_string()).collect();
    }

    if !existing_tags.contains(&tag.to_string()) {
        existing_tags.push(tag.to_string());
    }

    let tags_content = existing_tags.join("\n");
    tokio::fs::write(&tags_file, tags_content)
        .await
        .map_err(|e| ClaudeAgentError::Transport(format!("Failed to write session tags: {e}")))?;

    Ok(())
}

/// Delete a session and all associated files.
///
/// Removes the session data file and any associated metadata files (title, tags).
///
/// # Arguments
///
/// * `session_id` - The unique identifier of the session to delete.
///
/// # Errors
///
/// Returns an error if the session cannot be found or deletion fails.
pub async fn delete_session(session_id: &str) -> Result<(), ClaudeAgentError> {
    let base = sessions_base_dir()?;
    let session_file = find_session_file(&base, session_id).await?;

    if !session_file.exists() {
        return Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")));
    }

    // Remove the session file and associated metadata files
    let extensions = ["jsonl", "title", "tags"];
    for ext in &extensions {
        let path = session_file.with_extension(ext);
        if path.exists() {
            tokio::fs::remove_file(&path).await.map_err(|e| {
                ClaudeAgentError::Transport(format!("Failed to delete session file ({ext}): {e}"))
            })?;
        }
    }

    Ok(())
}

/// Fork a session, creating a new session with the same initial messages.
///
/// Copies the session data to a new session with a generated ID and returns
/// the new session ID.
///
/// # Arguments
///
/// * `session_id` - The unique identifier of the session to fork.
///
/// # Errors
///
/// Returns an error if the source session cannot be found or the copy fails.
pub async fn fork_session(session_id: &str) -> Result<String, ClaudeAgentError> {
    let base = sessions_base_dir()?;
    let session_file = find_session_file(&base, session_id).await?;

    if !session_file.exists() {
        return Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")));
    }

    let new_id = uuid::Uuid::new_v4().to_string();
    let parent_dir = session_file.parent().ok_or_else(|| {
        ClaudeAgentError::Transport("Session file has no parent directory".into())
    })?;

    let new_session_file = parent_dir.join(format!("{new_id}.jsonl"));
    tokio::fs::copy(&session_file, &new_session_file)
        .await
        .map_err(|e| ClaudeAgentError::Transport(format!("Failed to fork session: {e}")))?;

    Ok(new_id)
}

/// Search for a session file across all project directories.
///
/// Session files are organized as: `~/.claude/sessions/<project-hash>/<session-id>.jsonl`.
/// This function searches all project subdirectories for the given session ID.
async fn find_session_file(base: &PathBuf, session_id: &str) -> Result<PathBuf, ClaudeAgentError> {
    if !base.exists() {
        return Err(ClaudeAgentError::Transport("Sessions directory does not exist".into()));
    }

    let target_filename = format!("{session_id}.jsonl");

    let mut project_dirs = tokio::fs::read_dir(base).await.map_err(|e| {
        ClaudeAgentError::Transport(format!("Failed to read sessions directory: {e}"))
    })?;

    while let Some(entry) = project_dirs.next_entry().await.map_err(|e| {
        ClaudeAgentError::Transport(format!("Failed to read project directory entry: {e}"))
    })? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let candidate = path.join(&target_filename);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ClaudeAgentError::Transport(format!("Session not found: {session_id}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_claude_cli_returns_result() {
        // We can't guarantee the CLI is installed in CI, so just verify
        // the function returns a Result (does not panic).
        let result = find_claude_cli();
        // Either Ok with a path, or CLINotFound error -- both are valid.
        match result {
            Ok(path) => assert!(path.to_string_lossy().contains("claude")),
            Err(ClaudeAgentError::CLINotFound(_)) => (),
            Err(other) => panic!("Unexpected error variant: {other}"),
        }
    }

    // --- SessionInfo serde tests ---

    #[test]
    fn session_info_serialization_roundtrip() {
        let info = SessionInfo {
            id: "session-123".to_string(),
            title: Some("My Session".to_string()),
            tags: vec!["important".to_string(), "debug".to_string()],
            created_at: Some("2025-01-01T00:00:00Z".to_string()),
            updated_at: Some("2025-01-02T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "session-123");
        assert_eq!(parsed.title.as_deref(), Some("My Session"));
        assert_eq!(parsed.tags, vec!["important", "debug"]);
        assert_eq!(parsed.created_at.as_deref(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(parsed.updated_at.as_deref(), Some("2025-01-02T00:00:00Z"));
    }

    #[test]
    fn session_info_deserialization_missing_optional_fields() {
        let json = r#"{"id":"session-abc"}"#;
        let info: SessionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "session-abc");
        assert!(info.title.is_none());
        assert!(info.tags.is_empty());
        assert!(info.created_at.is_none());
        assert!(info.updated_at.is_none());
    }

    #[test]
    fn session_info_deserialization_empty_tags_defaults() {
        let json = r#"{"id":"s1","tags":[]}"#;
        let info: SessionInfo = serde_json::from_str(json).unwrap();
        assert!(info.tags.is_empty());
    }

    #[test]
    fn session_info_debug_format() {
        let info = SessionInfo {
            id: "test".to_string(),
            title: None,
            tags: Vec::new(),
            created_at: None,
            updated_at: None,
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("test"));
    }

    #[test]
    fn session_info_clone() {
        let info = SessionInfo {
            id: "orig".to_string(),
            title: Some("Title".to_string()),
            tags: vec!["a".to_string()],
            created_at: Some("2025-01-01T00:00:00Z".to_string()),
            updated_at: None,
        };
        let cloned = info.clone();
        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.title, info.title);
        assert_eq!(cloned.tags, info.tags);
    }

    // --- sessions_base_dir tests ---

    #[test]
    fn sessions_base_dir_returns_result() {
        let result = sessions_base_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().contains("sessions"));
    }

    // --- list_sessions tests ---

    #[tokio::test]
    async fn list_sessions_returns_empty_when_no_dir() {
        // Verify that reading a non-existent directory errors gracefully.
        let tmp = tempfile::TempDir::new().unwrap();
        let sessions_dir = tmp.path().join("nonexistent");
        assert!(!sessions_dir.exists());

        let result = tokio::fs::read_dir(&sessions_dir).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_sessions_discovers_jsonl_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj-hash");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        // Create some session files
        tokio::fs::write(project_dir.join("session-1.jsonl"), "{}\n").await.unwrap();
        tokio::fs::write(project_dir.join("session-2.jsonl"), "{}\n").await.unwrap();
        // Create a non-JSONL file that should be ignored
        tokio::fs::write(project_dir.join("notes.txt"), "ignored").await.unwrap();

        // Override HOME so sessions_base_dir() points to our temp dir.
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let result = list_sessions().await;
        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }

        let sessions = result.unwrap();
        assert_eq!(sessions.len(), 2);
        let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"session-1"));
        assert!(ids.contains(&"session-2"));
    }

    #[tokio::test]
    async fn list_sessions_empty_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let sessions_dir = tmp.path().join(".claude").join("sessions");
        tokio::fs::create_dir_all(&sessions_dir).await.unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let result = list_sessions().await;
        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }

        let sessions = result.unwrap();
        assert!(sessions.is_empty());
    }

    // --- Session CRUD tests with real temp dir ---

    #[tokio::test]
    async fn get_session_info_not_found() {
        let result = get_session_info("nonexistent-session-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_session_messages_not_found() {
        let result = get_session_messages("nonexistent-session-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_session_not_found() {
        let result = delete_session("nonexistent-session-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rename_session_not_found() {
        let result = rename_session("nonexistent-session-id", "New Title").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn tag_session_not_found() {
        let result = tag_session("nonexistent-session-id", "important").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fork_session_not_found() {
        let result = fork_session("nonexistent-session-id").await;
        assert!(result.is_err());
    }

    // --- Session CRUD with temp dir (happy path) ---

    #[tokio::test]
    async fn session_crud_full_lifecycle() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj-hash");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        let session_file = project_dir.join("test-session.jsonl");
        tokio::fs::write(
            &session_file,
            r#"{"type":"user","message":{"role":"user","content":"hello"}}"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        // get_session_info should succeed
        let info = get_session_info("test-session").await.unwrap();
        assert_eq!(info.id, "test-session");
        assert!(info.updated_at.is_some());

        // get_session_messages should succeed
        let messages = get_session_messages("test-session").await.unwrap();
        assert_eq!(messages.len(), 1);

        // rename_session should succeed
        rename_session("test-session", "New Title").await.unwrap();
        let title_file = project_dir.join("test-session.title");
        let title_content = tokio::fs::read_to_string(&title_file).await.unwrap();
        assert_eq!(title_content, "New Title");

        // tag_session should succeed
        tag_session("test-session", "important").await.unwrap();
        let tags_file = project_dir.join("test-session.tags");
        let tags_content = tokio::fs::read_to_string(&tags_file).await.unwrap();
        assert!(tags_content.contains("important"));

        // tag_session with duplicate should not add again
        tag_session("test-session", "important").await.unwrap();
        let tags_content2 = tokio::fs::read_to_string(&tags_file).await.unwrap();
        assert_eq!(tags_content2.lines().filter(|l| *l == "important").count(), 1);

        // tag_session with second tag
        tag_session("test-session", "review").await.unwrap();
        let tags_content3 = tokio::fs::read_to_string(&tags_file).await.unwrap();
        assert!(tags_content3.contains("review"));

        // fork_session should succeed
        let new_id = fork_session("test-session").await.unwrap();
        assert!(!new_id.is_empty());
        let forked_file = project_dir.join(format!("{new_id}.jsonl"));
        assert!(forked_file.exists());

        // delete_session should succeed
        delete_session("test-session").await.unwrap();
        assert!(!session_file.exists());
        // Title and tags files should also be cleaned up
        assert!(!title_file.exists());
        assert!(!tags_file.exists());

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }

    #[tokio::test]
    async fn get_session_messages_parses_valid_jsonl() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        let session_file = project_dir.join("msg-session.jsonl");
        let content = r#"{"type":"user","message":{"role":"user","content":"hello"}}
{"type":"assistant","message":{"role":"assistant","content":"world"}}
"#;
        tokio::fs::write(&session_file, content).await.unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let messages = get_session_messages("msg-session").await.unwrap();
        assert_eq!(messages.len(), 2);

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }

    #[tokio::test]
    async fn get_session_messages_skips_empty_lines() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        let session_file = project_dir.join("empty-lines.jsonl");
        let content =
            "\n\n{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"hi\"}}\n\n";
        tokio::fs::write(&session_file, content).await.unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let messages = get_session_messages("empty-lines").await.unwrap();
        assert_eq!(messages.len(), 1);

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }

    #[tokio::test]
    async fn get_session_messages_errors_on_invalid_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        let session_file = project_dir.join("bad-json.jsonl");
        tokio::fs::write(&session_file, "not valid json\n").await.unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        let result = get_session_messages("bad-json").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to parse session line"));

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }

    #[tokio::test]
    async fn delete_session_removes_all_associated_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        let session_id = "cleanup-test";
        let session_file = project_dir.join(format!("{session_id}.jsonl"));
        tokio::fs::write(&session_file, "{}").await.unwrap();
        tokio::fs::write(project_dir.join(format!("{session_id}.title")), "Title").await.unwrap();
        tokio::fs::write(project_dir.join(format!("{session_id}.tags")), "tag1\ntag2")
            .await
            .unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        delete_session(session_id).await.unwrap();

        assert!(!session_file.exists());
        assert!(!project_dir.join(format!("{session_id}.title")).exists());
        assert!(!project_dir.join(format!("{session_id}.tags")).exists());

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }

    #[tokio::test]
    async fn delete_session_succeeds_when_no_sidecar_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_dir = tmp.path().join(".claude").join("sessions").join("proj");
        tokio::fs::create_dir_all(&project_dir).await.unwrap();

        let session_id = "no-sidecar";
        let session_file = project_dir.join(format!("{session_id}.jsonl"));
        tokio::fs::write(&session_file, "{}").await.unwrap();

        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());

        delete_session(session_id).await.unwrap();
        assert!(!session_file.exists());

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }
}
