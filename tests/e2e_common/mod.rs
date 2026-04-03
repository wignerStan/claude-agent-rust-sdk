#![allow(dead_code)]

//! Shared E2E test helpers.
//!
//! All E2E tests require:
//! - `claude` CLI installed and in PATH.
//! - Valid `ANTHROPIC_API_KEY` or compatible auth token in environment.
//!
//! Run with: `cargo test -p claude-agent-api --test e2e_* -- --ignored`

use std::time::Duration;

use claude_agent::types::message::{ContentBlock, Message};
use claude_agent::types::ClaudeAgentError;
use claude_agent::types::ClaudeAgentOptions;
use futures::StreamExt;

/// Maximum timeout for slow operations (tool use, MCP).
pub const SLOW_TIMEOUT: Duration = Duration::from_secs(300);

/// Standard timeout for simple queries.
pub const STANDARD_TIMEOUT: Duration = Duration::from_secs(60);

/// Build `ClaudeAgentOptions` from environment.
///
/// Passes through authentication-related env vars to the Claude subprocess.
pub fn live_options() -> ClaudeAgentOptions {
    let mut opts = ClaudeAgentOptions::default();
    for var in &[
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
    ] {
        if let Ok(val) = std::env::var(var) {
            opts.env.insert(var.to_string(), val);
        }
    }
    if let Ok(path) = std::env::var("CLAUDE_CLI_PATH") {
        opts.cli_path = Some(std::path::PathBuf::from(path));
    }
    opts
}

/// Initialize tracing subscriber (idempotent — safe to call multiple times).
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// Collect messages from a stream until a `Result` message or timeout.
///
/// Returns all collected messages. Logs a warning if the timeout is reached.
pub async fn collect_until_result(
    stream: &mut (impl StreamExt<Item = Result<Message, ClaudeAgentError>> + Unpin),
    max: Duration,
) -> Vec<Message> {
    let mut messages = Vec::new();
    let result = tokio::time::timeout(max, async {
        while let Some(res) = stream.next().await {
            match res {
                Ok(msg) => {
                    let is_end = matches!(&msg, Message::Result(_));
                    messages.push(msg);
                    if is_end {
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("Stream error during collection: {e}");
                    break;
                },
            }
        }
    })
    .await;
    if result.is_err() {
        eprintln!("WARN: E2E collection timed out after {:?}", max);
    }
    messages
}

/// Extract all text content from a list of messages.
///
/// Concatenates text from all `ContentBlock::Text` blocks in `Message::Assistant` variants.
pub fn extract_text(messages: &[Message]) -> String {
    let mut text = String::new();
    for msg in messages {
        if let Message::Assistant(m) = msg {
            for block in &m.content {
                if let ContentBlock::Text(t) = block {
                    text.push_str(&t.text);
                }
            }
        }
    }
    text
}
