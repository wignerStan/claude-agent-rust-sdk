//! Streaming JSON message parser for Claude Code CLI.
//!
//! This module provides a streaming parser that reads from an `AsyncRead` source
//! (typically stdout from a subprocess) and parses individual JSON messages.
//!
//! # Features
//!
//! - **Streaming Parser**: Uses `serde_json::Deserializer::from_str` with `into_iter`
//!   for efficient streaming without loading entire input into memory
//! - **Split Packet Handling**: Correctly handles JSON objects split across
//!   multiple read operations
//! - **Multiple Messages Per Line**: Supports multiple JSON objects on a single line
//! - **Buffer Management**: Configurable buffer size with overflow protection
//! - **Error Context**: Includes buffer preview in parse errors for debugging
//!
//! # Architecture
//!
//! The parser maintains an internal buffer that accumulates data from the underlying
//! reader. It attempts to parse complete JSON objects using a streaming deserializer.
//! If parsing fails due to incomplete data, it continues reading. If parsing
//! fails due to invalid JSON, it returns an error with a buffer preview.
//!
//! # Example
//!
//! See `MessageReader::new()` and `MessageReader::with_capacity()` for usage examples.
//!
//! # Buffer Overflow Protection
//!
//! The parser will return an error if the internal buffer exceeds `max_buffer_size`.
//! This prevents memory exhaustion from malformed input or unbounded data streams.
//! The default buffer size is 64KB, which can be customized using
//! `MessageReader::with_capacity()`.

use claude_agent_types::ClaudeAgentError;
use futures::Stream;
use pin_project_lite::pin_project;
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, BufReader};

// Default buffer size 64KB
const DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

pin_project! {
    /// A stream reader that parses JSON messages from an AsyncRead source.
    ///
    /// Handles split packets, multiple messages per line, and messy buffering.
    ///
    /// # Thread Safety
    ///
    /// This type is `Unpin` and implements `Stream`, making it safe to use
    /// across multiple async tasks. The internal state is managed through
    /// pin projection.
    ///
    /// # Buffer Management
    ///
    /// The reader maintains an internal buffer that grows as data is read.
    /// If the buffer exceeds `max_buffer_size`, a `Transport` error is returned
    /// to prevent memory exhaustion.
    ///
    /// # Error Handling
    ///
    /// - **EOF**: Returns `Poll::Ready(None)` when the underlying reader is exhausted
    /// - **Incomplete JSON**: Continues reading if JSON is incomplete (EOF error)
    /// - **Invalid JSON**: Returns error with buffer preview (first 100 chars) for debugging
    /// - **Buffer Overflow**: Returns error if buffer size limit is exceeded
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tokio::io::AsyncRead;
    /// use claude_agent_transport::reader::MessageReader;
    /// use futures::StreamExt;
    ///
    /// let stdout = /* your AsyncRead source */;
    /// let reader = MessageReader::new(stdout);
    /// let mut stream = Box::pin(reader);
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(msg) => println!("Parsed: {}", msg),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub struct MessageReader<R> {
        #[pin]
        reader: BufReader<R>,
        buffer: String,
        max_buffer_size: usize,
    }
}

impl<R: AsyncRead> MessageReader<R> {
    /// Create a new message reader with default buffer size (64KB).
    ///
    /// # Parameters
    ///
    /// - `inner`: The underlying async read source (e.g., stdout from subprocess)
    ///
    /// # Buffer Size
    ///
    /// The default buffer size is 64KB. Use `with_capacity()` to customize
    /// for high-throughput scenarios or to reduce memory usage.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use claude_agent_transport::reader::MessageReader;
    /// use tokio::io::AsyncRead;
    ///
    /// let stdout = /* get stdout from subprocess */;
    /// let reader = MessageReader::new(stdout);
    /// ```
    pub fn new(inner: R) -> Self {
        Self {
            reader: BufReader::new(inner),
            buffer: String::new(),
            max_buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }

    /// Create a new message reader with custom buffer size.
    ///
    /// # Parameters
    ///
    /// - `inner`: The underlying async read source
    /// - `max_size`: Maximum buffer size in bytes before returning overflow error
    ///
    /// # Buffer Overflow
    ///
    /// If the buffer grows beyond `max_size`, subsequent reads will return
    /// a `Transport` error with buffer overflow message. This prevents memory
    /// exhaustion from malformed input.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use claude_agent_transport::reader::MessageReader;
    /// use tokio::io::AsyncRead;
    ///
    /// let stdout = /* get stdout from subprocess */;
    /// // Create reader with 1MB buffer for high-throughput scenario
    /// let reader = MessageReader::with_capacity(stdout, 1024 * 1024);
    /// ```
    pub fn with_capacity(inner: R, max_size: usize) -> Self {
        Self { reader: BufReader::new(inner), buffer: String::new(), max_buffer_size: max_size }
    }
}

impl<R: AsyncRead + Unpin> Stream for MessageReader<R> {
    type Item = Result<Value, ClaudeAgentError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // 1. Check if we have a complete message in buffer using streaming deserializer
            // We use from_str which allows us to peek without consuming if we clone mechanism,
            // but effectively we just try to parse the first object.

            // We need a specific block to limit borrow scope if needed
            {
                let mut stream =
                    serde_json::Deserializer::from_str(this.buffer).into_iter::<Value>();
                match stream.next() {
                    Some(Ok(val)) => {
                        let offset = stream.byte_offset();
                        // Consumed one object
                        this.buffer.drain(..offset);
                        return Poll::Ready(Some(Ok(val)));
                    },
                    Some(Err(ref e)) if e.is_eof() => {
                        // Incomplete JSON, need more data.
                        // Break out of match to read more.
                    },
                    Some(Err(e)) => {
                        // Syntax error or other error
                        // If it's just "trailing characters", that shouldn't happen with into_iter (it handles stream).
                        // If it's truly invalid syntax, we should return error.
                        // But wait: if buffer is "{" (eof), it hits is_eof().
                        // If buffer is "invalid", it hits here.
                        let preview = this.buffer.chars().take(100).collect::<String>();
                        return Poll::Ready(Some(Err(ClaudeAgentError::JSONDecode(format!(
                            "Parse error: {}. Buffer preview: {}",
                            e, preview
                        )))));
                    },
                    None => {
                        // Buffer might be empty or just whitespace
                        if this.buffer.trim().is_empty() {
                            this.buffer.clear();
                            // If buffer empty, we need read.
                        } else {
                            // Should theoretically not happen if trim is not empty and next() is None?
                            // Actually it means only whitespace.
                            this.buffer.clear();
                        }
                    },
                }
            }

            // 2. Read more data
            let mut buf = [0u8; 1024];
            let mut read_buf = tokio::io::ReadBuf::new(&mut buf);

            match this.reader.as_mut().poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(())) => {
                    let n = read_buf.filled().len();
                    if n == 0 {
                        // EOF
                        if !this.buffer.trim().is_empty() {
                            // Try parse remaining
                            match serde_json::from_str(this.buffer) {
                                Ok(val) => {
                                    this.buffer.clear();
                                    return Poll::Ready(Some(Ok(val)));
                                },
                                Err(e) => {
                                    return Poll::Ready(Some(Err(ClaudeAgentError::JSONDecode(
                                        format!("EOF with invalid json: {}", e),
                                    ))));
                                },
                            }
                        }
                        return Poll::Ready(None);
                    }

                    let chunk = String::from_utf8_lossy(read_buf.filled());
                    this.buffer.push_str(&chunk);

                    if this.buffer.len() > *this.max_buffer_size {
                        return Poll::Ready(Some(Err(ClaudeAgentError::Transport(
                            "Buffer overflow".to_string(),
                        ))));
                    }
                    // Loop back to try parsing
                },
                Poll::Ready(Err(e)) => {
                    return Poll::Ready(Some(Err(ClaudeAgentError::Transport(e.to_string()))))
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use serde_json::json;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_reader_single_json_object() {
        let data = json!({"type": "message", "content": "hello"});
        let reader = Cursor::new(format!("{}\n", data).into_bytes());
        let mut stream = MessageReader::new(reader);

        let result = stream.next().await.expect("expected a value");
        let val = result.expect("expected Ok");
        assert_eq!(val["type"], "message");
        assert_eq!(val["content"], "hello");

        // Stream should end after one value
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_reader_multiple_json_objects() {
        let msg1 = json!({"id": 1});
        let msg2 = json!({"id": 2});
        let data = format!("{}\n{}\n", msg1, msg2);

        let reader = Cursor::new(data.into_bytes());
        let mut stream = MessageReader::new(reader);

        let val1 = stream.next().await.expect("expected value 1").expect("expected Ok");
        assert_eq!(val1["id"], 1);

        let val2 = stream.next().await.expect("expected value 2").expect("expected Ok");
        assert_eq!(val2["id"], 2);

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_reader_empty_input() {
        let reader = Cursor::new(Vec::<u8>::new());
        let mut stream = MessageReader::new(reader);

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_reader_incomplete_json() {
        // Incomplete JSON: no closing brace, followed by EOF
        let reader = Cursor::new(b"{\"type\": ".to_vec());
        let mut stream = MessageReader::new(reader);

        let result = stream.next().await.expect("expected a result");
        assert!(result.is_err(), "expected error for incomplete JSON");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("EOF with invalid json") || err.contains("Parse error"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_reader_invalid_json() {
        // Completely invalid JSON
        let reader = Cursor::new(b"this is not json\n".to_vec());
        let mut stream = MessageReader::new(reader);

        let result = stream.next().await.expect("expected a result");
        assert!(result.is_err(), "expected error for invalid JSON");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Parse error"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn test_reader_whitespace_only() {
        let reader = Cursor::new(b"   \n  \t\n  ".to_vec());
        let mut stream = MessageReader::new(reader);

        // Whitespace-only input should yield None (EOF)
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_reader_trailing_newline() {
        let msg = json!({"type": "result", "status": "ok"});
        let data = format!("{}\n\n", msg);

        let reader = Cursor::new(data.into_bytes());
        let mut stream = MessageReader::new(reader);

        let val = stream.next().await.expect("expected value").expect("expected Ok");
        assert_eq!(val["type"], "result");

        // After the trailing newline, stream should end
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_with_capacity_small_buffer() {
        let data = json!({"type": "test"}).to_string();
        let reader = Cursor::new(data.into_bytes());
        // Very small buffer -- data is small enough to fit
        let mut stream = MessageReader::with_capacity(reader, 16);

        let val = stream.next().await.expect("expected msg").expect("parse ok");
        assert_eq!(val["type"], "test");
    }

    #[tokio::test]
    async fn test_buffer_overflow_protection() {
        // Create data larger than the tiny buffer
        let large_data = "x".repeat(256);
        let reader = Cursor::new(large_data.into_bytes());
        let mut stream = MessageReader::with_capacity(reader, 128);

        let result = stream.next().await;
        assert!(result.is_some());
        let item = result.expect("expected Some");
        assert!(item.is_err(), "buffer overflow should return error");
        let err = item.unwrap_err().to_string();
        assert!(err.contains("Buffer overflow"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn test_json_with_embedded_newlines_in_string() {
        // JSON string values with escaped \n should parse correctly
        let data = r#"{"type":"message","content":"Line 1\nLine 2"}"#;
        let reader = Cursor::new(data.as_bytes());
        let mut stream = MessageReader::new(reader);

        let val = stream.next().await.expect("expected msg").expect("parse ok");
        assert_eq!(val["type"], "message");
        assert_eq!(val["content"], "Line 1\nLine 2");
    }

    #[tokio::test]
    async fn test_multiple_objects_no_newline() {
        // Multiple JSON objects concatenated without newline separators
        let msg1 = json!({"id": 1});
        let msg2 = json!({"id": 2});
        let data = format!("{}{}", msg1, msg2);

        let reader = Cursor::new(data.into_bytes());
        let mut stream = MessageReader::new(reader);

        let v1 = stream.next().await.expect("expected first").expect("parse ok");
        assert_eq!(v1["id"], 1);

        let v2 = stream.next().await.expect("expected second").expect("parse ok");
        assert_eq!(v2["id"], 2);

        assert!(stream.next().await.is_none());
    }
}
