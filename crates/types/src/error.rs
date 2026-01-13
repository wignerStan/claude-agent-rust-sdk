use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ClaudeAgentError {
    #[error("CLI not found: {0}")]
    CLINotFound(String),

    #[error("CLI connection error: {0}")]
    CLIConnection(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("JSON decode error: {0}")]
    JSONDecode(String),

    #[error("Message parse error: {0}")]
    MessageParse(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Control protocol error: {0}")]
    ControlProtocol(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
