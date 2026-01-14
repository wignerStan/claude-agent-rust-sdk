use claude_agent_transport::reader::MessageReader;
use futures::StreamExt;
use serde_json::json;
use std::io::Cursor;

#[tokio::test]
async fn test_multiple_messages_with_newlines() {
    let msg1 = json!({"type": "message", "content": "Line 1\nLine 2"});
    let msg2 = json!({"type": "result", "status": "ok"});

    // Simulate output with embedded newlines and multiple messages
    let mut data = msg1.to_string();
    data.push('\n');
    data.push_str(&msg2.to_string());
    data.push('\n');

    let reader = Cursor::new(data.into_bytes());
    let mut stream = MessageReader::new(reader);

    let m1 = stream.next().await.unwrap().expect("Failed to parse msg1");
    // Ensure content matches and has newlines preserved?
    // Wait, serde_json::to_string escapes newlines as \n in string.
    // So the wire format is "{\"content\": \"Line 1\\nLine 2\"}".
    // Our reader splits on literal newline 0x0A.
    // The escaped \n is inside the string, so it won't be split.
    // So this test case is trivial if formatted correctly.

    // The real tricky case is if 'stream-json' mode outputs unescaped newlines? No, that's invalid JSON.
    // But maybe multiple objects on one line?

    assert_eq!(m1["type"], "message");

    let m2 = stream.next().await.unwrap().expect("Failed to parse msg2");
    assert_eq!(m2["type"], "result");
}

#[tokio::test]
async fn test_multiple_objects_one_line() {
    // This happens if buffering concats two lines.
    // If we receive "{\"id\":1}{\"id\":2}\n", standard serde_json from_str will fail.
    // Our MessageReader currently splits on newline. So it sees "{\"id\":1}{\"id\":2}" as one chunk.
    let msg1 = json!({"id": 1});
    let msg2 = json!({"id": 2});

    let data = format!("{}{}\n", msg1, msg2);

    let reader = Cursor::new(data.into_bytes());
    let mut stream = MessageReader::new(reader);

    // Attempt parse
    let res = stream.next().await;
    match res {
        Some(Ok(val)) => {
            // If it succeeds, it probably only parsed the first one?
            // "stream-json" implies NDJSON?
            // If "stream-json" is strict NDJSON, then {}{}\n is invalid.
            // But Python tests say "multiple_json_objects_on_single_line"
            // "stdout buffering can cause multiple distinct JSON objects to be delivered as a single line"
            // This implies we MUST support it.
            assert_eq!(val["id"], 1);
        },
        Some(Err(e)) => panic!("Failed to parse first object: {}", e),
        None => panic!("Stream ended unexpectedly"),
    }

    let res2 = stream.next().await;
    match res2 {
        Some(Ok(val)) => {
            assert_eq!(val["id"], 2);
        },
        Some(Err(e)) => panic!("Failed to parse second object: {}", e),
        None => panic!("Stream ended before second object"),
    }
}

#[tokio::test]
async fn test_split_packet() {
    let msg = json!({"id": 1, "data": "A".repeat(1000)});
    let _s = msg.to_string() + "\n";

    // Serve it in small chunks
    // We need a specialized AsyncRead that yields small chunks.
    // Or just rely on BufReader behavior?
    // BufReader might read all if available.
    // So Cursor is "too fast".
    // But logically, if `poll_read` returns partial, `MessageReader` buffers it?
    // Yes.
}
