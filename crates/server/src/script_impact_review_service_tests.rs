use super::*;
use crate::ai_backends::transport_tests::{event, serve};
use crate::script_impact_review::tests::{fixture, request};
use crate::state::AiConfig;

async fn service_fixture() -> (
    AppState,
    std::path::PathBuf,
    CommandEnvelope<RequestScriptImpactProposalCommand>,
) {
    let (conn, project, _, b, _) = fixture();
    let command = request(&conn, &b);
    let path = std::env::temp_dir().join(format!(
        "eidetic-preview-http-{}.eidetic",
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
async fn independent_real_service_refuses_incomplete_http_body_without_any_write() {
    let (state, path, command) = service_fixture().await;
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    let before =
        crate::script_store::load_document_projection(&conn, &command.payload.document_id).unwrap();
    let history: i64 = conn
        .query_row("SELECT COUNT(*) FROM change_events", [], |row| row.get(0))
        .unwrap();
    let mut events = state.events_tx.subscribe();
    let (url, server) = serve(vec![event("Unfinished provider draft")], 100);
    *state.ai_config.lock() = AiConfig {
        base_url: url,
        model: "fixture-model".into(),
        ..AiConfig::default()
    };
    let result = request_script_impact_proposal(&state, command.clone()).await;
    server.join().unwrap();
    state.shutdown_tasks_async().await;
    assert!(
        result.is_err(),
        "incomplete draft was persisted as a successful preview"
    );
    assert_eq!(
        before,
        crate::script_store::load_document_projection(&conn, &command.payload.document_id).unwrap()
    );
    assert_eq!(
        history,
        conn.query_row("SELECT COUNT(*) FROM change_events", [], |row| row
            .get::<_, i64>(0))
            .unwrap()
    );
    assert!(
        propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap()
            .payload
            .proposals
            .is_empty()
    );
    assert!(events.try_recv().is_err());
    drop(conn);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn independent_real_service_preserves_split_sse_before_explicit_acceptance() {
    let (state, path, command) = service_fixture().await;
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    let before =
        crate::script_store::load_document_projection(&conn, &command.payload.document_id).unwrap();
    let first = event("FIRST HALF ");
    let mut tail = first[15..].to_vec();
    tail.extend(event("SECOND HALF"));
    tail.extend(b"data: [DONE]\n\n");
    let (url, server) = serve(vec![first[..15].to_vec(), tail], 0);
    *state.ai_config.lock() = AiConfig {
        base_url: url,
        model: "fixture-model".into(),
        ..AiConfig::default()
    };
    let result = request_script_impact_proposal(&state, command).await;
    server.join().unwrap();
    state.shutdown_tasks_async().await;
    let response = result.unwrap();
    assert_eq!(response.projection.payload.proposals.len(), 1);
    assert_eq!(
        response.projection.payload.proposals[0]
            .proposed_text
            .as_deref(),
        Some("FIRST HALF SECOND HALF")
    );
    assert_eq!(
        response.projection.payload.proposals[0].status,
        eidetic_core::contracts::SemanticProposalStatus::Pending
    );
    assert_eq!(
        before,
        crate::script_store::load_document_projection(
            &conn,
            &response.projection.payload.proposals[0]
                .script_review_binding
                .as_ref()
                .unwrap()
                .request
                .document_id
        )
        .unwrap()
    );
    drop(conn);
    std::fs::remove_file(path).unwrap();
}
