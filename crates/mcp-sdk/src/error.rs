//! Error types for the MCP SDK.

use thiserror::Error;

/// Errors that can occur during MCP SDK operations.
#[derive(Debug, Error)]
pub enum McpSdkError {
    /// A requested tool was not found in the registry.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// Tool input failed validation.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A tool handler returned an error.
    #[error("handler error: {0}")]
    HandlerError(String),

    /// An I/O error occurred during stdio communication.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialization or deserialization error occurred.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_not_found_display() {
        let err = McpSdkError::ToolNotFound("my_tool".to_string());
        assert_eq!(err.to_string(), "tool not found: my_tool");
    }

    #[test]
    fn test_invalid_input_display() {
        let err = McpSdkError::InvalidInput("missing field 'name'".to_string());
        assert_eq!(err.to_string(), "invalid input: missing field 'name'");
    }

    #[test]
    fn test_handler_error_display() {
        let err = McpSdkError::HandlerError("timeout".to_string());
        assert_eq!(err.to_string(), "handler error: timeout");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        let err = McpSdkError::from(io_err);
        assert!(err.to_string().contains("pipe broke"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err = McpSdkError::from(json_err);
        assert!(err.to_string().contains("JSON error"));
    }
}
