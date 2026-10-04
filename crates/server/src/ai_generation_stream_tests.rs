use super::stream_generated_text;
use eidetic_core::Error;
use eidetic_core::ai::backend::GenerateStream;
use futures::{executor::block_on, stream};

fn tokens(items: Vec<Result<String, Error>>) -> GenerateStream {
    Box::pin(stream::iter(items))
}

#[test]
fn partial_output_followed_by_error_is_failure_and_stops_progress() {
    let mut progress = Vec::new();
    let result = block_on(stream_generated_text(
        tokens(vec![
            Ok("unfinished prefix".into()),
            Err(Error::AiBackend("broken stream".into())),
            Ok("must never be consumed".into()),
        ]),
        |token, count| progress.push((token, count)),
    ));
    assert!(matches!(result, Err(Error::AiBackend(message)) if message == "broken stream"));
    assert_eq!(progress, vec![("unfinished prefix".to_string(), 1)]);
}

#[test]
fn error_before_output_remains_failure() {
    let mut progress = Vec::new();
    let result = block_on(stream_generated_text(
        tokens(vec![Err(Error::AiBackend("unavailable".into()))]),
        |token, count| progress.push((token, count)),
    ));
    assert!(matches!(result, Err(Error::AiBackend(message)) if message == "unavailable"));
    assert!(progress.is_empty());
}

#[test]
fn successful_eof_preserves_text_and_progress_order() {
    let mut progress = Vec::new();
    let text = block_on(stream_generated_text(
        tokens(vec![Ok("Ada ".into()), Ok("enters.\n".into())]),
        |token, count| progress.push((token, count)),
    ))
    .unwrap();
    assert_eq!(text, "Ada enters.\n");
    assert_eq!(
        progress,
        vec![("Ada ".to_string(), 1), ("enters.\n".to_string(), 2)]
    );
}

#[test]
fn empty_eof_is_successful_collection_with_no_text() {
    let text = block_on(stream_generated_text(tokens(vec![]), |_, _| {
        panic!("empty EOF must not emit progress");
    }))
    .unwrap();
    assert!(text.is_empty());
}

#[test]
fn empty_tokens_do_not_turn_empty_output_into_content() {
    let mut progress = Vec::new();
    let text = block_on(stream_generated_text(
        tokens(vec![Ok(String::new())]),
        |token, count| {
            progress.push((token, count));
        },
    ))
    .unwrap();
    assert!(text.is_empty());
    assert_eq!(progress, vec![(String::new(), 1)]);
}
