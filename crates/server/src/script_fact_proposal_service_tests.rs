//! Real service with labelled synthetic local HTTP streams; no model execution.
use super::*;
use crate::ai_backends::transport_tests::{event, serve};
use crate::script_fact_proposal::tests::{VALUE, fixture};
use crate::state::AiConfig;

async fn setup() -> (
    AppState,
    std::path::PathBuf,
    CommandEnvelope<RequestScriptFactProposalCommand>,
) {
    let (conn, project, _, _, _, command) = fixture();
    let path = std::env::temp_dir().join(format!(
        "eidetic-fact-http-{}.eidetic",
        uuid::Uuid::new_v4()
    ));
    conn.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    let state = AppState::new().await;
    *state.project.lock() = Some(project);
    state.project_database.set_active_path(path.clone());
    (state, path, command)
}

#[tokio::test]
async fn split_synthetic_stream_creates_only_pending_fact_proposal_and_exact_retry_skips_provider()
{
    let (state, path, command) = setup().await;
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    let before =
        crate::script_store::load_document_projection(&conn, &command.payload.document_id).unwrap();
    let json=serde_json::json!({"value":VALUE,"rationale":"Synthetic local HTTP fixture; no real-model quality claim"}).to_string();
    let cut = json.len() / 2;
    let (url, server) = serve(
        vec![
            event(&json[..cut]),
            event(&json[cut..]),
            b"data: [DONE]\n\n".to_vec(),
        ],
        0,
    );
    *state.ai_config.lock() = AiConfig {
        base_url: url,
        model: "synthetic-fixture".into(),
        ..AiConfig::default()
    };
    let response = request_script_fact_proposal(&state, command.clone())
        .await
        .unwrap();
    server.join().unwrap();
    let proposal = &response.projection.payload.proposals[0];
    assert_eq!(proposal.status, SemanticProposalStatus::Pending);
    assert_eq!(
        proposal.proposed_value,
        Some(FieldValue::Text(VALUE.into()))
    );
    assert_eq!(
        before,
        crate::script_store::load_document_projection(&conn, &command.payload.document_id).unwrap()
    );
    state.ai_config.lock().base_url = "http://127.0.0.1:1/v1".into();
    let replay = request_script_fact_proposal(&state, command).await.unwrap();
    assert_eq!(
        replay.outcome,
        history_store::RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(replay.projection, response.projection);
    state.shutdown_tasks_async().await;
    drop(conn);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn incomplete_or_target_injecting_synthetic_output_never_creates_fact_proposal() {
    for text in [
        "{\"value\":\"Unfinished",
        "{\"value\":\"Blue\",\"rationale\":\"Synthetic\",\"target\":\"Eli\"}",
    ] {
        let (state, path, command) = setup().await;
        let conn = crate::sqlite::open_write_connection(&path).unwrap();
        let history: i64 = conn
            .query_row("SELECT count(*) FROM change_events", [], |r| r.get(0))
            .unwrap();
        let (url, server) = serve(vec![event(text), b"data: [DONE]\n\n".to_vec()], 0);
        *state.ai_config.lock() = AiConfig {
            base_url: url,
            model: "synthetic-fixture".into(),
            ..AiConfig::default()
        };
        assert!(request_script_fact_proposal(&state, command).await.is_err());
        server.join().unwrap();
        assert_eq!(
            history,
            conn.query_row("SELECT count(*) FROM change_events", [], |r| r
                .get::<_, i64>(0))
                .unwrap()
        );
        assert!(
            propagation_proposal_store::load_propagation_proposals(&conn)
                .unwrap()
                .is_empty()
        );
        state.shutdown_tasks_async().await;
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }
}
