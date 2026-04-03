//! Query function for one-shot interactions.

use futures::stream::BoxStream;

use crate::core::ClaudeAgent;
use crate::types::{ClaudeAgentError, ClaudeAgentOptions, Message};

/// Query Claude Code for one-shot or unidirectional streaming interactions.
///
/// This function is ideal for simple, stateless queries where you don't need
/// bidirectional communication or conversation management. For interactive,
/// stateful conversations, use `ClaudeAgentClient` instead.
///
/// # Arguments
///
/// * `prompt` - The prompt to send to Claude.
/// * `options` - Optional configuration for the query.
///
/// # Returns
///
/// A stream of messages from Claude.
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent::api::query;
/// use claude_agent::types::ClaudeAgentOptions;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let mut stream = query("What is 2+2?", None).await.unwrap();
///     while let Some(result) = stream.next().await {
///         match result {
///             Ok(message) => println!("{:?}", message),
///             Err(e) => eprintln!("Error: {}", e),
///         }
///     }
/// }
/// ```
pub async fn query(
    prompt: &str,
    options: Option<ClaudeAgentOptions>,
) -> Result<BoxStream<'static, Result<Message, ClaudeAgentError>>, ClaudeAgentError> {
    let opts = options.unwrap_or_default();
    let mut agent = ClaudeAgent::new(opts);

    agent.connect(Some(prompt)).await?;

    // Note: This is a simplified implementation. In practice, we'd need to
    // properly manage the agent lifetime and ensure cleanup.
    let stream = agent.query(prompt).await?;

    // Convert to 'static lifetime by collecting and re-streaming
    // This is a workaround for the borrow checker
    let messages: Vec<_> = futures::StreamExt::collect::<Vec<_>>(stream).await;
    Ok(Box::pin(futures::stream::iter(messages)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The `query()` function spawns a real subprocess (claude CLI).
    // These tests are ignored because they require an interactive CLI session
    // and hang in CI/automated environments.

    #[tokio::test]
    #[ignore]
    async fn query_fails_without_valid_transport() {
        let result = query("test prompt", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn query_with_options_fails_without_valid_transport() {
        let opts = ClaudeAgentOptions::default();
        let result = query("test prompt", Some(opts)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn query_error_message_is_descriptive() {
        let result = query("hello", None).await;
        match result {
            Err(e) => assert!(!e.to_string().is_empty(), "Error message should not be empty"),
            Ok(_) => panic!("Expected query to fail without valid transport"),
        }
    }
}
