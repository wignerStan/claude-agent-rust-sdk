use claude_agent_transport::SubprocessTransport;
use claude_agent_types::ClaudeAgentOptions;

#[tokio::test]
async fn test_transport_instantiation() {
    let options = ClaudeAgentOptions::default();
    let _transport = SubprocessTransport::new(Some("test".to_string()), options);
    // We can't easily test connect() here without a real Claude binary or complex mocking
    // but at least we verify the public API compiles and runs.
}
