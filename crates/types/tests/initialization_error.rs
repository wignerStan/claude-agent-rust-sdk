use claude_agent_types::ClaudeAgentError;

#[test]
fn test_initialization_error() {
    let error = ClaudeAgentError::Initialization("Failed to initialize component".to_string());
    assert!(error.to_string().contains("Failed to initialize component"));
    assert!(error.to_string().contains("Initialization error"));
}
