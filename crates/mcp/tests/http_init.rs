use claude_agent_mcp::transports::HttpMcpServer;

#[tokio::test]
async fn test_http_server_creation_returns_result() {
    let server_result = HttpMcpServer::new("test".to_string(), "http://localhost:8080".to_string());
    assert!(server_result.is_ok());
}
