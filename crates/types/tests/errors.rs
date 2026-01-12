use claude_agent_types::ClaudeAgentError;

#[test]
fn test_cli_not_found_error() {
    let error = ClaudeAgentError::CLINotFound("Claude Code not found".to_string());
    assert!(error.to_string().contains("Claude Code not found"));
    assert!(error.to_string().contains("CLI not found"));
}

#[test]
fn test_connection_error() {
    let error = ClaudeAgentError::CLIConnection("Failed to connect to CLI".to_string());
    assert!(error.to_string().contains("Failed to connect to CLI"));
    assert!(error.to_string().contains("CLI connection error"));
}

#[test]
fn test_process_error() {
    // Note: Rust implementation currently simplifies ProcessError to a String
    let error = ClaudeAgentError::Process("Process failed: exit code 1".to_string());
    assert!(error.to_string().contains("Process failed"));
    assert!(error.to_string().contains("exit code 1"));
    assert!(error.to_string().contains("Process error"));
}

#[test]
fn test_json_decode_error() {
    let error = ClaudeAgentError::JSONDecode("Failed to decode JSON".to_string());
    assert!(error.to_string().contains("Failed to decode JSON"));
    assert!(error.to_string().contains("JSON decode error"));
}

#[test]
fn test_unknown_error() {
    let error = ClaudeAgentError::Unknown("Something weird happened".to_string());
    assert!(error.to_string().contains("Something weird happened"));
    assert!(error.to_string().contains("Unknown error"));
}
