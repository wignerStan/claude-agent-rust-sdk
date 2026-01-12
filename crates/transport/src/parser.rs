//! JSON message parsing for Claude Code CLI output.

use claude_agent_types::ClaudeAgentError;
use serde_json::Value;

/// Parse a single line of JSON from CLI output.
pub fn parse_line(line: &str) -> Result<Value, ClaudeAgentError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(ClaudeAgentError::JSONDecode("Empty line".to_string()));
    }

    serde_json::from_str(trimmed)
        .map_err(|e| ClaudeAgentError::JSONDecode(format!("JSON parse error: {}", e)))
}

/// Check if a JSON value represents a result message (end of response).
pub fn is_result_message(value: &Value) -> bool {
    value.get("type").and_then(|t| t.as_str()) == Some("result")
}

/// Extract message type from a JSON value.
pub fn get_message_type(value: &Value) -> Option<&str> {
    value.get("type").and_then(|t| t.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_valid() {
        let json = r#"{"type": "assistant", "content": "Hello"}"#;
        let result = parse_line(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_line_empty() {
        let result = parse_line("");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_result_message() {
        let result_msg: Value = serde_json::json!({"type": "result"});
        let other_msg: Value = serde_json::json!({"type": "assistant"});

        assert!(is_result_message(&result_msg));
        assert!(!is_result_message(&other_msg));
    }
}
