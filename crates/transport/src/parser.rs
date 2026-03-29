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

    #[test]
    fn test_parse_line_whitespace_only() {
        let result = parse_line("   \t  ");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Empty line"), "unexpected error: {err}");
    }

    #[test]
    fn test_parse_line_invalid_json() {
        let result = parse_line("not json at all");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("JSON parse error"), "unexpected error: {err}");
    }

    #[test]
    fn test_parse_line_with_leading_trailing_whitespace() {
        let json = r#"  {"type": "user"}  "#;
        let result = parse_line(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["type"], "user");
    }

    #[test]
    fn test_is_result_message_no_type_field() {
        let msg: Value = serde_json::json!({"content": "hello"});
        assert!(!is_result_message(&msg));
    }

    #[test]
    fn test_is_result_message_non_string_type() {
        let msg: Value = serde_json::json!({"type": 42});
        assert!(!is_result_message(&msg));
    }

    #[test]
    fn test_get_message_type() {
        let msg: Value = serde_json::json!({"type": "assistant", "content": "hi"});
        assert_eq!(get_message_type(&msg), Some("assistant"));
    }

    #[test]
    fn test_get_message_type_no_type_field() {
        let msg: Value = serde_json::json!({"content": "hi"});
        assert!(get_message_type(&msg).is_none());
    }

    #[test]
    fn test_get_message_type_non_string_type() {
        let msg: Value = serde_json::json!({"type": 123});
        assert!(get_message_type(&msg).is_none());
    }
}
