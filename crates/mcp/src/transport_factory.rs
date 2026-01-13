//! Transport factory for creating MCP server transports based on configuration.

use std::sync::Arc;
use std::time::Duration;

use claude_agent_types::config::{McpServerConfig, McpTransportType};
use claude_agent_types::ClaudeAgentError;

use crate::manager::McpServer;
use crate::transports::{HttpMcpServer, SseMcpServer, StdioMcpServer};

/// Creates an MCP server transport based on the provided configuration.
///
/// # Parameters
///
/// - `name`: Unique identifier for the server
/// - `config`: Server configuration specifying transport type and settings
///
/// # Returns
///
/// Returns an `Arc<dyn McpServer>` that can be registered with `McpServerManager`.
///
/// # Errors
///
/// Returns `ClaudeAgentError::Config` if the configuration is invalid.
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent_mcp::transport_factory::create_mcp_server;
/// use claude_agent_types::config::{McpServerConfig, McpTransportType};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let config = McpServerConfig {
///     transport: McpTransportType::Http,
///     url: Some("http://localhost:8080".to_string()),
///     ..Default::default()
/// };
///
/// let server = create_mcp_server("my_server".to_string(), config)?;
/// Ok(())
/// }
/// ```
pub fn create_mcp_server(
    name: String,
    config: McpServerConfig,
) -> Result<Arc<dyn McpServer>, ClaudeAgentError> {
    match config.transport {
        McpTransportType::Stdio => create_stdio_server(name, config),
        McpTransportType::Http => create_http_server(name, config),
        McpTransportType::Sse => create_sse_server(name, config),
        McpTransportType::Auto => create_auto_server(name, config),
    }
}

fn create_stdio_server(
    name: String,
    config: McpServerConfig,
) -> Result<Arc<dyn McpServer>, ClaudeAgentError> {
    let command = config.command.ok_or_else(|| {
        ClaudeAgentError::Config("Stdio transport requires 'command' field".to_string())
    })?;

    Ok(Arc::new(StdioMcpServer::new(name, command, config.args)))
}

fn create_http_server(
    name: String,
    config: McpServerConfig,
) -> Result<Arc<dyn McpServer>, ClaudeAgentError> {
    let url = config.url.ok_or_else(|| {
        ClaudeAgentError::Config("HTTP transport requires 'url' field".to_string())
    })?;

    let server = if let Some(timeout) = config.timeout_secs {
        HttpMcpServer::with_timeout(name, url, Duration::from_secs(timeout))
    } else {
        HttpMcpServer::new(name, url)
    };

    Ok(Arc::new(server))
}

fn create_sse_server(
    name: String,
    config: McpServerConfig,
) -> Result<Arc<dyn McpServer>, ClaudeAgentError> {
    let url = config.url.ok_or_else(|| {
        ClaudeAgentError::Config("SSE transport requires 'url' field".to_string())
    })?;

    let server = if let Some(timeout) = config.timeout_secs {
        SseMcpServer::with_timeout(name, url, Duration::from_secs(timeout))
    } else {
        SseMcpServer::new(name, url)
    };

    Ok(Arc::new(server))
}

fn create_auto_server(
    name: String,
    config: McpServerConfig,
) -> Result<Arc<dyn McpServer>, ClaudeAgentError> {
    // Auto mode: prefer HTTP if URL is provided, otherwise use stdio
    if config.url.is_some() {
        // Try HTTP first
        create_http_server(name, config)
    } else if config.command.is_some() {
        // Fall back to stdio
        create_stdio_server(name, config)
    } else {
        Err(ClaudeAgentError::Config(
            "Auto transport requires either 'url' (for HTTP) or 'command' (for Stdio)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_http_server() {
        let config = McpServerConfig {
            transport: McpTransportType::Http,
            url: Some("http://localhost:8080".to_string()),
            timeout_secs: Some(60),
            ..Default::default()
        };

        let server = create_mcp_server("test".to_string(), config).unwrap();
        assert_eq!(server.name(), "test");
    }

    #[test]
    fn test_create_sse_server() {
        let config = McpServerConfig {
            transport: McpTransportType::Sse,
            url: Some("http://localhost:8080/sse".to_string()),
            ..Default::default()
        };

        let server = create_mcp_server("sse_test".to_string(), config).unwrap();
        assert_eq!(server.name(), "sse_test");
    }

    #[test]
    fn test_create_stdio_server() {
        let config = McpServerConfig {
            transport: McpTransportType::Stdio,
            command: Some("python".to_string()),
            args: vec!["-m".to_string(), "mcp_server".to_string()],
            ..Default::default()
        };

        let server = create_mcp_server("stdio_test".to_string(), config).unwrap();
        assert_eq!(server.name(), "stdio_test");
    }

    #[test]
    fn test_create_auto_server_with_url() {
        let config = McpServerConfig {
            transport: McpTransportType::Auto,
            url: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };

        let server = create_mcp_server("auto_http".to_string(), config).unwrap();
        assert_eq!(server.name(), "auto_http");
    }

    #[test]
    fn test_create_auto_server_with_command() {
        let config = McpServerConfig {
            transport: McpTransportType::Auto,
            command: Some("node".to_string()),
            args: vec!["server.js".to_string()],
            ..Default::default()
        };

        let server = create_mcp_server("auto_stdio".to_string(), config).unwrap();
        assert_eq!(server.name(), "auto_stdio");
    }

    #[test]
    fn test_http_server_missing_url() {
        let config = McpServerConfig {
            transport: McpTransportType::Http,
            ..Default::default()
        };

        let result = create_mcp_server("test".to_string(), config);
        assert!(result.is_err());
    }

    #[test]
    fn test_stdio_server_missing_command() {
        let config = McpServerConfig {
            transport: McpTransportType::Stdio,
            ..Default::default()
        };

        let result = create_mcp_server("test".to_string(), config);
        assert!(result.is_err());
    }
}
