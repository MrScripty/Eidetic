use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use futures::StreamExt;

use super::*;

#[derive(Clone, Copy)]
enum Provider {
    LlamaCpp,
    OpenRouter,
}

// Raw HTTP deliberately controls framing and Content-Length. No external
// provider, model, credential, or network policy change is involved.
pub(crate) fn serve(parts: Vec<Vec<u8>>, extra_length: usize) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = socket.read(&mut buffer).unwrap();
            assert_ne!(count, 0);
            request.extend_from_slice(&buffer[..count]);
            if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]).to_lowercase();
                let length: usize = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                if request.len() >= end + 4 + length {
                    assert!(headers.starts_with("post /v1/chat/completions http/1.1"));
                    let body: serde_json::Value =
                        serde_json::from_slice(&request[end + 4..]).unwrap();
                    assert_eq!(body["stream"], true);
                    break;
                }
            }
        }
        let length = parts.iter().map(Vec::len).sum::<usize>() + extra_length;
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n").unwrap();
        for part in parts {
            socket.write_all(&part).unwrap();
            socket.flush().unwrap();
            thread::sleep(Duration::from_millis(20));
        }
    });
    (url, handle)
}

pub(crate) fn event(text: &str) -> Vec<u8> {
    format!(
        "data: {}\n\n",
        serde_json::json!({"choices": [{"delta": {"content": text}}]})
    )
    .into_bytes()
}

async fn generate(
    provider: Provider,
    url: String,
    prompt: ChatPrompt,
) -> Result<GenerateStream, Error> {
    let config = AiConfig {
        backend_type: match provider {
            Provider::LlamaCpp => BackendType::LlamaCpp,
            Provider::OpenRouter => BackendType::OpenRouter,
        },
        base_url: url.clone(),
        model: "fixture-model".into(),
        api_key: Some("loopback-fixture".into()),
        ..AiConfig::default()
    };
    match provider {
        Provider::LlamaCpp => {
            Backend::from_config(&config)
                .generate(&prompt, &config)
                .await
        }
        Provider::OpenRouter => {
            openrouter::OpenRouterBackend::new(&config)
                .generate_at_url(&prompt, &config, &format!("{url}/chat/completions"))
                .await
        }
    }
}

fn prompt() -> ChatPrompt {
    ChatPrompt {
        system: "fixture".into(),
        user: "fixture".into(),
    }
}

#[tokio::test]
async fn real_adapters_preserve_split_events_unicode_order_and_clean_completion() {
    for provider in [Provider::LlamaCpp, Provider::OpenRouter] {
        let first = event("FIRST HALF 雨 ");
        let unicode = first
            .windows(3)
            .position(|bytes| bytes == "雨".as_bytes())
            .unwrap();
        let mut tail = first[unicode + 1..].to_vec();
        tail.extend(event("SECOND HALF"));
        tail.extend(b"data: [DONE]\r\n\r\n");
        let (url, server) = serve(
            vec![first[..9].to_vec(), first[9..unicode + 1].to_vec(), tail],
            0,
        );
        let mut stream = generate(provider, url, prompt()).await.unwrap();
        let mut tokens = Vec::new();
        while let Some(token) = stream.next().await {
            tokens.push(token.unwrap());
        }
        server.join().unwrap();
        assert_eq!(tokens, ["FIRST HALF 雨 ", "SECOND HALF"]);
    }
}

#[tokio::test]
async fn real_adapters_refuse_truncated_body_missing_done_and_error_frames() {
    let prefix = event("Unfinished provider draft");
    for provider in [Provider::LlamaCpp, Provider::OpenRouter] {
        for (tail, extra_length) in [
            (Vec::new(), 100),
            (Vec::new(), 0),
            (b"data: [DONE]\n\n".to_vec(), 100),
            (
                b"data: {\"error\":{\"message\":\"provider failed\"}}\n\n".to_vec(),
                0,
            ),
            (b"data: malformed\n\n".to_vec(), 0),
            (b"data: \xff\n\n".to_vec(), 0),
            (b"data: {\"choices\": []}".to_vec(), 0),
            (b"data: {\ndata: \"choices\": []}\n".to_vec(), 0),
            (b"data: [DONE]\n\ndata: unfinished".to_vec(), 0),
        ] {
            let (url, server) = serve(vec![prefix.clone(), tail], extra_length);
            let mut stream = generate(provider, url, prompt()).await.unwrap();
            let mut failed = false;
            while let Some(token) = stream.next().await {
                if token.is_err() {
                    failed = true;
                }
            }
            server.join().unwrap();
            assert!(failed, "partial output was incorrectly reported complete");
        }
    }
}

#[tokio::test]
async fn real_adapter_failure_preserves_authored_text_and_creates_no_proposal_or_history() {
    for provider in [Provider::LlamaCpp, Provider::OpenRouter] {
        let (mut conn, _, _, b, _) = crate::script_impact_review::tests::fixture();
        let command = crate::script_impact_review::tests::request(&conn, &b);
        let binding = crate::script_impact_review::capture(&conn, &command.payload).unwrap();
        let before = crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
        let history: i64 = conn
            .query_row("SELECT COUNT(*) FROM change_events", [], |row| row.get(0))
            .unwrap();
        let (url, server) = serve(vec![event("Unfinished provider draft")], 100);
        let result = crate::script_impact_prompt::preview_with_provider(&binding, |prompt| {
            generate(provider, url, prompt)
        })
        .await;
        // The service records only a successful complete preview.
        if let Ok(text) = result {
            crate::script_impact_review::record_proposal(&mut conn, &command, binding, text, 30)
                .unwrap();
            panic!("incomplete transport was allowed to persist a proposal");
        }
        server.join().unwrap();
        assert_eq!(
            before,
            crate::script_store::load_document_projection(&conn, &b.document_id).unwrap()
        );
        assert_eq!(
            history,
            conn.query_row("SELECT COUNT(*) FROM change_events", [], |row| row
                .get::<_, i64>(0))
                .unwrap()
        );
        let projection =
            crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
                .unwrap();
        assert!(projection.payload.proposals.is_empty());
    }
}

#[tokio::test]
async fn oversized_real_provider_streams_preserve_canonical_text_and_history() {
    let oversized_line = vec![b'x'; super::sse::MAX_LINE_BYTES + 1];
    let mut oversized_event = Vec::new();
    for _ in 0..=super::sse::MAX_EVENT_BYTES / (64 * 1024) {
        oversized_event.extend_from_slice(b"data: ");
        oversized_event.extend(vec![b' '; 64 * 1024]);
        oversized_event.push(b'\n');
    }
    for provider in [Provider::LlamaCpp, Provider::OpenRouter] {
        for body in [&oversized_line, &oversized_event] {
            let (mut conn, _, _, b, _) = crate::script_impact_review::tests::fixture();
            let command = crate::script_impact_review::tests::request(&conn, &b);
            let binding = crate::script_impact_review::capture(&conn, &command.payload).unwrap();
            let before =
                crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
            let history: i64 = conn
                .query_row("SELECT COUNT(*) FROM change_events", [], |row| row.get(0))
                .unwrap();
            let (url, server) = serve(vec![event("partial"), body.clone()], 0);
            let result = crate::script_impact_prompt::preview_with_provider(&binding, |prompt| {
                generate(provider, url, prompt)
            })
            .await;
            match result {
                Ok(text) => {
                    crate::script_impact_review::record_proposal(
                        &mut conn, &command, binding, text, 30,
                    )
                    .unwrap();
                    panic!("oversized provider stream was allowed to persist a proposal");
                }
                Err(error) => assert!(error.to_string().contains("exceeds size limit")),
            }
            server.join().unwrap();
            assert_eq!(
                before,
                crate::script_store::load_document_projection(&conn, &b.document_id).unwrap()
            );
            assert_eq!(
                history,
                conn.query_row("SELECT COUNT(*) FROM change_events", [], |row| row
                    .get::<_, i64>(0))
                    .unwrap()
            );
            assert!(
                crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
                    .unwrap()
                    .payload
                    .proposals
                    .is_empty()
            );
        }
    }
}

#[tokio::test]
async fn real_adapters_save_the_entire_split_preview_for_explicit_review() {
    for provider in [Provider::LlamaCpp, Provider::OpenRouter] {
        let (mut conn, _, _, b, _) = crate::script_impact_review::tests::fixture();
        let command = crate::script_impact_review::tests::request(&conn, &b);
        let binding = crate::script_impact_review::capture(&conn, &command.payload).unwrap();
        let before = crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
        let first = event("FIRST HALF ");
        let mut tail = first[15..].to_vec();
        tail.extend(event("SECOND HALF"));
        tail.extend(b"data: [DONE]\n\n");
        let (url, server) = serve(vec![first[..15].to_vec(), tail], 0);
        let text = crate::script_impact_prompt::preview_with_provider(&binding, |prompt| {
            generate(provider, url, prompt)
        })
        .await
        .unwrap();
        server.join().unwrap();
        assert_eq!(text, "FIRST HALF SECOND HALF");
        crate::script_impact_review::record_proposal(
            &mut conn,
            &command,
            binding,
            text.clone(),
            30,
        )
        .unwrap();
        let projection =
            crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
                .unwrap();
        assert_eq!(projection.payload.proposals.len(), 1);
        assert_eq!(
            projection.payload.proposals[0].proposed_text.as_deref(),
            Some(text.as_str())
        );
        assert_eq!(
            before,
            crate::script_store::load_document_projection(&conn, &b.document_id).unwrap()
        );
    }
}

#[tokio::test]
async fn collect_full_refuses_real_transport_failure_and_accepts_clean_stream() {
    for extra_length in [0, 100] {
        let mut body = event("complete draft");
        body.extend(b"data: [DONE]\n\n");
        let (url, server) = serve(vec![body], extra_length);
        let config = AiConfig {
            base_url: url,
            model: "fixture-model".into(),
            ..AiConfig::default()
        };
        let result = Backend::from_config(&config)
            .generate_full(&prompt(), &config)
            .await;
        server.join().unwrap();
        if extra_length == 0 {
            assert_eq!(result.unwrap(), "complete draft");
        } else {
            assert!(result.is_err());
        }
    }
}
