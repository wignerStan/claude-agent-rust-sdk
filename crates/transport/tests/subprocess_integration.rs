use claude_agent_transport::{SubprocessTransport, Transport};
use claude_agent_types::ClaudeAgentOptions;
use futures::StreamExt;
use std::path::PathBuf;

fn make_dummy_cli_options(cli_path: PathBuf) -> ClaudeAgentOptions {
    ClaudeAgentOptions { cli_path: Some(cli_path), ..Default::default() }
}

fn create_dummy_cli() -> PathBuf {
    // Use a unique temp directory per invocation to avoid "Text file busy" races
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("dummy_claude");

    // Write a script that echoes a JSON line and exits
    use std::io::Write;
    let mut file = std::fs::File::create(&path).expect("failed to open temp file");
    write!(file, "#!/bin/sh\necho '{{\"type\":\"result\",\"subtype\":\"success\"}}'\n")
        .expect("failed to write script");
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).expect("metadata failed").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("set_permissions failed");
    }

    // Leak the temp dir so the file persists for the test duration
    let path = path;
    std::mem::forget(dir);
    path
}

#[tokio::test]
async fn test_write_before_connect_returns_error() {
    let transport =
        SubprocessTransport::new(Some("test".to_string()), ClaudeAgentOptions::default());
    let result = transport.write("hello").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not connected"), "unexpected error: {err}");
}

#[tokio::test]
async fn test_read_before_connect_returns_error_stream() {
    let transport =
        SubprocessTransport::new(Some("test".to_string()), ClaudeAgentOptions::default());
    let mut stream = transport.read_messages().await;

    let result = stream.next().await;
    assert!(result.is_some(), "stream should yield one error item");
    let item = result.expect("expected Some");
    assert!(item.is_err(), "expected Err from unconnected transport");
    let err = item.unwrap_err().to_string();
    assert!(err.contains("not connected"), "unexpected error: {err}");
}

#[tokio::test]
async fn test_close_without_connect_succeeds() {
    let mut transport =
        SubprocessTransport::new(Some("test".to_string()), ClaudeAgentOptions::default());
    let result = transport.close().await;
    assert!(result.is_ok(), "close without connect should succeed");
}

#[tokio::test]
async fn test_connect_with_dummy_cli() {
    let cli_path = create_dummy_cli();
    let options = make_dummy_cli_options(cli_path);
    let mut transport = SubprocessTransport::new(Some("hello".to_string()), options);

    let result = Transport::connect(&mut transport).await;
    assert!(result.is_ok(), "connect should succeed: {:?}", result.err());

    // After connect, close should also succeed
    let close_result = transport.close().await;
    assert!(close_result.is_ok(), "close should succeed: {:?}", close_result.err());
}

#[tokio::test]
async fn test_close_is_idempotent() {
    let cli_path = create_dummy_cli();
    let options = make_dummy_cli_options(cli_path);
    let mut transport = SubprocessTransport::new(Some("test".to_string()), options);

    Transport::connect(&mut transport).await.expect("connect failed");
    transport.close().await.expect("first close failed");
    // Second close should succeed without error
    let result = transport.close().await;
    assert!(result.is_ok(), "second close should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_write_and_read_after_connect() {
    let cli_path = create_dummy_cli();
    let options = make_dummy_cli_options(cli_path);
    let mut transport = SubprocessTransport::new(Some("test".to_string()), options);

    Transport::connect(&mut transport).await.expect("connect failed");

    // Write should succeed after connect
    let write_result = transport.write("hello").await;
    assert!(write_result.is_ok(), "write should succeed after connect");

    transport.close().await.expect("close failed");
}
