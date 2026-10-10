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
        pumas_request: None,
        started: false,
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

#[test]
fn pumas_typed_stream_checks_order_contract_selection_and_correlation() {
    let typed = || {
        let mut reader = reader();
        reader.pumas_request = Some(("request".into(), "model".into(), "profile".into()));
        reader
    };
    let started = br#"data: {"kind":"started","contract_version":1,"request_id":"request","capability":"chat_generation","model":"model","profile":"profile"}

"#;
    let delta = br#"data: {"kind":"delta","request_id":"request","text":"hello"}

"#;
    let completed = br#"data: {"kind":"completed","request_id":"request","finish_reason":"stop"}

"#;
    let mut reader = typed();
    assert!(feed(&mut reader, started).unwrap().is_empty());
    assert_eq!(feed(&mut reader, delta).unwrap(), ["hello"]);
    assert!(feed(&mut reader, completed).unwrap().is_empty());
    assert!(reader.completed);
    assert!(feed(&mut reader, delta).is_err());
    assert!(feed(&mut typed(), delta).is_err());
    assert!(feed(&mut typed(), completed).is_err());
    for replacement in [
        String::from_utf8_lossy(started)
            .replace("\"contract_version\":1", "\"contract_version\":2"),
        String::from_utf8_lossy(started).replace("\"model\":\"model\"", "\"model\":\"other\""),
        String::from_utf8_lossy(started)
            .replace("\"profile\":\"profile\"", "\"profile\":\"other\""),
        String::from_utf8_lossy(started)
            .replace("\"request_id\":\"request\"", "\"request_id\":\"other\""),
    ] {
        assert!(feed(&mut typed(), replacement.as_bytes()).is_err());
    }
    let mut reader = typed();
    feed(&mut reader, started).unwrap();
    assert!(feed(&mut reader, br#"data: {"kind":"failed","request_id":"request","error":{"code":"transport_lost","outcome":"unknown"}}

"#).unwrap_err().to_string().contains("transport_lost"));
}

#[tokio::test]
async fn dropping_pumas_stream_closes_pending_transport_without_fabricating_completion() {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let peer = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 4096];
        assert!(socket.read(&mut request).await.unwrap() > 0);
        let body = concat!(
            "data: {\"kind\":\"started\",\"contract_version\":1,\"request_id\":\"request\",\"capability\":\"chat_generation\",\"model\":\"model\",\"profile\":\"profile\"}\n\n",
            "data: {\"kind\":\"delta\",\"request_id\":\"request\",\"text\":\"partial\"}\n\n"
        );
        // A declared unfinished body keeps the producer connection open.
        socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{body}",body.len()+100).as_bytes()).await.unwrap();
        let mut byte = [0];
        tokio::time::timeout(std::time::Duration::from_secs(5), socket.read(&mut byte))
            .await
            .unwrap()
            .unwrap()
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(url)
        .send()
        .await
        .unwrap();
    let mut stream = pumas_tokens(response, "request".into(), "model".into(), "profile".into());
    assert_eq!(stream.next().await.unwrap().unwrap(), "partial");
    drop(stream);
    assert_eq!(peer.await.unwrap(), 0);
}
