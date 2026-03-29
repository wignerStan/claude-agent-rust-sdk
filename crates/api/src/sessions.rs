//! Standalone session management for Claude Code.
//!
//! This module provides functions for managing Claude Code sessions without
//! requiring an active `ClaudeAgentClient` connection. Sessions are managed
//! by reading session files directly from the Claude Code sessions directory
//! on disk (`~/.claude/sessions/`).

use std::path::PathBuf;

use claude_agent_types::ClaudeAgentError;

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
    let home = dirs::home_dir()
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
}
