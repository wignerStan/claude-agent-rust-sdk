use claude_agent_mcp::transports::SseMcpServer;

#[tokio::test]
async fn test_sse_server_creation_returns_result() {
    let server_result =
        SseMcpServer::new("test".to_string(), "http://localhost:3000/sse".to_string());
    assert!(server_result.is_ok());
}
