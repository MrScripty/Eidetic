use crate::script_context_scope::tests::{fixture, node, request, target};
use crate::state::{AiConfig, AppState, ServerEvent};
use crate::{ai_service, command_service, persistence, projection_service, sqlite};
use eidetic_core::contracts::*;

#[tokio::test]
async fn native_scene_order_command_refreshes_complete_window_and_reviews_only_on_acceptance() {
    let (source, project, blocks) = fixture();
    let b = &blocks[1];
    let directory =
        std::env::temp_dir().join(format!("eidetic-scene-window-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("project.db");
    source.execute_batch("CREATE TABLE episode_structure (id INTEGER PRIMARY KEY CHECK (id=1), template_name TEXT NOT NULL, segments_json TEXT NOT NULL);").unwrap();
    source
        .execute(
            "INSERT INTO episode_structure VALUES (1,?1,?2)",
            rusqlite::params![
                project.timeline.structure.template_name,
                serde_json::to_string(&project.timeline.structure.segments).unwrap()
            ],
        )
        .unwrap();
    source
        .execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let state = AppState::new().await;
    *state.project.lock() = Some(project.clone());
    state.project_database.set_active_path(path.clone());
    let mut generation =
        eidetic_core::ai::prompt::build_generate_request(&project, node(b)).unwrap();
    ai_service::attach_ai_generation_context(&mut generation, path.clone(), node(b))
        .await
        .unwrap();
    let scope = generation.script_context_scope.as_ref().unwrap();
    assert_eq!(
        scope
            .segment_ids
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        [
            "segment.A",
            "segment.F",
            "segment.B",
            "segment.C",
            "segment.D"
        ]
    );
    let before = projection_service::script_document_projection(
        &state,
        projection_service::ScriptDocumentProjectionRequest {
            document_id: b.document_id.clone(),
        },
    )
    .await
    .unwrap();
    let mut events = state.events_tx.subscribe();
    command_service::set_timeline_node_range(
        &state,
        crate::timeline_script_placement::tests::command(&blocks[5], 3000, 3500),
    )
    .await
    .unwrap();
    assert!(matches!(
        events.try_recv().unwrap(),
        ServerEvent::TimelineChanged
    ));
    assert!(matches!(
        events.try_recv().unwrap(),
        ServerEvent::ScriptChanged
    ));
    let after = projection_service::script_document_projection(
        &state,
        projection_service::ScriptDocumentProjectionRequest {
            document_id: b.document_id.clone(),
        },
    )
    .await
    .unwrap();
    assert!(after.version.0 > before.version.0);
    let conn = sqlite::open_write_connection(&path).unwrap();
    let reviewed = target(&conn, b);
    assert_eq!(reviewed.blocks[0].block.text, "  Retained human B — 雨\n\n");
    assert_eq!(
        reviewed.impact.as_ref().unwrap().causes[0].reason,
        ScriptImpactReason::ContextChanged
    );
    let command = request(&conn, b);
    let mut frame = crate::ai_backends::transport_tests::event("Synthetic scene-order preview\n\n");
    frame.extend(b"data: [DONE]\n\n");
    let (url, provider) = crate::ai_backends::transport_tests::serve(vec![frame], 0);
    *state.ai_config.lock() = AiConfig {
        base_url: url,
        model: "synthetic-fixture-model".into(),
        ..AiConfig::default()
    };
    crate::script_impact_review_service::request_script_impact_proposal(&state, command.clone())
        .await
        .unwrap();
    provider.join().unwrap();
    let proposal = crate::propagation_proposal_store::load_propagation_proposal(
        &conn,
        &command.payload.proposal_id,
    )
    .unwrap()
    .unwrap();
    assert_eq!(proposal.status, SemanticProposalStatus::Pending);
    assert_eq!(target(&conn, b), reviewed);
    let binding = proposal.script_review_binding.unwrap();
    assert_eq!(
        binding
            .script_inputs
            .iter()
            .map(|i| i.segment_id.as_str())
            .collect::<Vec<_>>(),
        [
            "segment.F",
            "segment.E",
            "segment.B",
            "segment.C",
            "segment.D"
        ]
    );
    command_service::accept_propagation_proposal(
        &state,
        CommandEnvelope::new(AcceptPropagationProposalCommand {
            proposal_id: command.payload.proposal_id,
        }),
    )
    .await
    .unwrap();
    assert_eq!(
        target(&conn, b).blocks[0].block.text,
        "Synthetic scene-order preview\n\n"
    );
    assert!(!target(&conn, b).impact.unwrap().needs_review);
    assert_eq!(
        target(&conn, &blocks[0]).blocks[0].block.text,
        "  Authored A — 雨\n\n"
    );
    state.shutdown_tasks_async().await;
    drop(conn);
    std::fs::remove_dir_all(directory).unwrap();
}
