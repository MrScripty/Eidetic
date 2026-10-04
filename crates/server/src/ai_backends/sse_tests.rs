use super::*;

fn reader() -> Reader {
    Reader {
        body: stream::empty().boxed(),
        chunk: Vec::new(),
        scan: 0,
        line: Vec::new(),
        data: Vec::new(),
        completed: false,
        provider: "fixture",
    }
}

fn feed(reader: &mut Reader, chunk: &[u8]) -> Result<Vec<String>, Error> {
    assert_eq!(reader.scan, reader.chunk.len());
    reader.chunk = chunk.to_vec();
    reader.scan = 0;
    let mut tokens = Vec::new();
    while let Some(token) = reader.lines()? {
        tokens.push(token);
    }
    Ok(tokens)
}

fn event(text: &str) -> Vec<u8> {
    format!(
        "data: {}\r\n\r\n",
        serde_json::json!({"choices":[{"delta":{"content":text}}]})
    )
    .into_bytes()
}

#[test]
fn fragmented_unicode_crlf_and_multiple_events_preserve_order() {
    let mut reader = reader();
    let mut body = event("FIRST 雨");
    body.extend(event("SECOND"));
    body.extend(b"data: [DONE]\r\n\r\n");
    let mut tokens = Vec::new();
    for byte in body {
        tokens.extend(feed(&mut reader, &[byte]).unwrap());
        assert_eq!(reader.scan, reader.chunk.len());
    }
    assert_eq!(tokens, ["FIRST 雨", "SECOND"]);
    assert!(reader.completed);
    assert!(reader.line.is_empty() && reader.data.is_empty());
}

#[test]
fn unterminated_line_accepts_limit_and_rejects_next_byte_before_retaining_it() {
    let mut reader = reader();
    for _ in 0..MAX_LINE_BYTES {
        feed(&mut reader, b"x").unwrap();
    }
    assert_eq!(reader.line.len(), MAX_LINE_BYTES);
    assert!(
        feed(&mut reader, b"x")
            .unwrap_err()
            .to_string()
            .contains("line exceeds")
    );
    assert_eq!(reader.line.len(), MAX_LINE_BYTES);
}

#[test]
fn complete_line_limit_is_checked_even_when_newline_is_in_same_chunk() {
    let mut reader = reader();
    let mut comment = vec![b':'; MAX_LINE_BYTES];
    comment.push(b'\n');
    feed(&mut reader, &comment).unwrap();
    assert!(reader.line.is_empty());
    comment.insert(0, b':');
    assert!(feed(&mut reader, &comment).is_err());
}

#[test]
fn event_accepts_exact_limit_then_rejects_another_data_line() {
    let mut reader = reader();
    // Six data lines, each within the line cap. Include their five separators.
    let data_size = (MAX_EVENT_BYTES - 5) / 6;
    for _ in 0..5 {
        let mut line = b"data: ".to_vec();
        line.extend(vec![b' '; data_size]);
        line.push(b'\n');
        feed(&mut reader, &line).unwrap();
    }
    let remaining = MAX_EVENT_BYTES - reader.data.len() - 1;
    let mut last = b"data: ".to_vec();
    last.extend(vec![b' '; remaining]);
    last.push(b'\n');
    feed(&mut reader, &last).unwrap();
    assert_eq!(reader.data.len(), MAX_EVENT_BYTES);
    assert!(
        feed(&mut reader, b"data: x\n")
            .unwrap_err()
            .to_string()
            .contains("event exceeds")
    );
    assert_eq!(reader.data.len(), MAX_EVENT_BYTES);
}

#[test]
fn valid_multiline_json_and_near_limit_payload_complete() {
    let mut reader = reader();
    let mut body =
        b"data: {\ndata: \"choices\": [{\"delta\": {\"content\": \"ok\"}}]}\n\ndata: ".to_vec();
    let payload = serde_json::json!({"choices":[{"delta":{"content":"long"}}]}).to_string();
    body.extend(vec![b' '; MAX_LINE_BYTES - 6 - payload.len()]);
    body.extend(payload.as_bytes());
    body.extend(b"\n\ndata: [DONE]\n\n");
    assert_eq!(feed(&mut reader, &body).unwrap(), ["ok", "long"]);
    assert!(reader.completed);
}

#[test]
fn malformed_utf8_json_and_data_after_done_are_errors() {
    for body in [
        b"data: \xff\n\n".as_slice(),
        b"data: {broken}\n\n",
        b"data: [DONE]\n\ndata: {}\n\n",
    ] {
        assert!(feed(&mut reader(), body).is_err());
    }
}

#[test]
fn provider_error_serialization_is_bounded_on_utf8_boundary() {
    let error = serde_json::json!({"error":{"message":"雨".repeat(2000)}});
    let message = feed(&mut reader(), format!("data: {error}\n\n").as_bytes())
        .unwrap_err()
        .to_string();
    assert!(message.contains("[truncated]"));
    assert!(message.contains("雨"));
    assert!(message.len() < MAX_ERROR_BYTES + 100);
}

#[test]
fn a_chunk_is_consumed_incrementally_without_a_token_queue() {
    let mut reader = reader();
    reader.chunk = [event("first"), event("second")].concat();
    assert_eq!(reader.lines().unwrap(), Some("first".into()));
    assert!(reader.scan < reader.chunk.len());
    let consumed = reader.scan;
    assert_eq!(reader.lines().unwrap(), Some("second".into()));
    assert!(reader.scan > consumed);
    assert_eq!(reader.lines().unwrap(), None);
}
