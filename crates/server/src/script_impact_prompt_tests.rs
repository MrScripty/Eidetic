use super::*;
use crate::script_impact_review::{
    capture,
    tests::{fixture, request},
};
use futures::stream;

#[tokio::test]
async fn deterministic_provider_receives_canonical_authored_evidence_and_resolved_context() {
    let (conn, _, _, b, _) = fixture();
    let command = request(&conn, &b);
    let binding = capture(&conn, &command.payload).unwrap();
    let output = preview_with_provider(&binding, |prompt| async move {
        assert!(
            prompt
                .user
                .contains("  A now carries a blue umbrella — 雨\n\n")
        );
        assert!(prompt.user.contains("TARGET BLOCK TO UPDATE:\nOriginal B"));
        assert!(
            prompt
                .system
                .contains("presentation placement is not fictional time")
        );
        let result: GenerateStream = Box::pin(stream::iter(vec![Ok("  Proposed B\n\n".into())]));
        Ok(result)
    })
    .await
    .unwrap();
    assert_eq!(output, "  Proposed B\n\n");
}

#[tokio::test]
async fn failed_or_empty_provider_output_never_returns_a_partial_preview() {
    for tokens in [
        vec![
            Ok("unfinished prefix".into()),
            Err(eidetic_core::Error::AiBackend("broken provider".into())),
        ],
        vec![],
        vec![Ok(" \n".into())],
    ] {
        let (conn, _, _, b, _) = fixture();
        let binding = capture(&conn, &request(&conn, &b).payload).unwrap();
        assert!(
            preview_with_provider(&binding, |_| async move {
                let result: GenerateStream = Box::pin(stream::iter(tokens));
                Ok(result)
            })
            .await
            .is_err()
        );
    }
}
