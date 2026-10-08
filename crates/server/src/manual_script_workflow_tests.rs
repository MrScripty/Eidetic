use super::*;
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::StoryLevel;

#[tokio::test]
async fn manual_edit_reaches_preview_and_generation_admission_with_a_stale_project_mirror() {
    let path = std::env::temp_dir().join(format!("eidetic-script-consumers-{}.db", Uuid::new_v4()));
    let mut project = eidetic_core::Template::MultiCam.build_project("Manual screenplay");
    let scene_ids: Vec<_> = project
        .timeline
        .nodes
        .iter()
        .filter(|node| node.level == StoryLevel::Scene)
        .take(2)
        .map(|node| node.id)
        .collect();
    assert_eq!(scene_ids.len(), 2);
    for (index, id) in scene_ids.iter().enumerate() {
        let node = project.timeline.node_mut(*id).unwrap();
        node.time_range.start_ms = index as u64 * 1000;
        node.time_range.end_ms = index as u64 * 1000 + 1000;
        node.content.notes = "Scene notes".into();
        node.content.content = "STALE MIRROR SCRIPT".into();
    }
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let mut conn = crate::sqlite::open_write_connection(&path).unwrap();
    let block_id = ScriptBlockId::new("screenplay.manual.first").unwrap();
    let seed = CommandEnvelope::new(SetScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        document_title: project.name.clone(),
        document_sort_order: 0,
        segment_id: ScriptSegmentId::new("screenplay.first").unwrap(),
        source_node_id: Some(scene_ids[0].0.to_string()),
        segment_start_ms: 0,
        segment_end_ms: 1000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 0,
        block_id: block_id.clone(),
        block_kind: ScriptBlockKind::Action,
        text: "Ada leaves in rain.".into(),
        span_provenance: ScriptSpanProvenance::AiGenerated,
        sort_order: 0,
    });
    let (_, seeded) =
        crate::script_document_command::apply_set_script_block(&mut conn, &seed, 10).unwrap();
    drop(conn);
    let state = AppState::new().await;
    *state.project.lock() = Some(project.clone());
    *state.project_path.lock() = Some(path.clone());
    let stack_request = ContextStackProjectionRequest {
        target_node_id: scene_ids[1],
    };
    let stack_before =
        crate::context_influence_service::context_stack_projection(&state, stack_request.clone())
            .await
            .unwrap();
    let saved = crate::command_service::edit_script_block(
        &state,
        CommandEnvelope::new(EditScriptBlockCommand {
            block_kind: None,
            document_id: seed.payload.document_id,
            block_id,
            expected_revision_event_id: seeded.payload.segments[0].blocks[0]
                .revision_event_id
                .unwrap(),
            text: "Ada leaves under clear skies.\nBen waves goodbye.".into(),
        }),
    )
    .await
    .unwrap();
    let saved: ProjectionEnvelope<ScriptDocumentProjection> =
        serde_json::from_value(serde_json::to_value(saved).unwrap()["projection"].clone()).unwrap();
    let stack_after =
        crate::context_influence_service::context_stack_projection(&state, stack_request)
            .await
            .unwrap();
    assert!(stack_after.version.0 > stack_before.version.0);
    let evidence = stack_after.payload.script_context.unwrap();
    let first = evidence
        .iter()
        .find(|input| input.block_id.as_str() == "screenplay.manual.first")
        .unwrap();
    assert_eq!(
        first.text,
        "Ada leaves under clear skies.\nBen waves goodbye."
    );
    assert_eq!(
        Some(first.revision_event_id),
        saved.payload.segments[0].blocks[0].revision_event_id
    );
    assert!(
        !evidence
            .iter()
            .any(|input| input.text.contains("STALE MIRROR SCRIPT"))
    );
    let preview = preview_ai_context(&state, scene_ids[1].0).await.unwrap();
    assert!(
        preview
            .user
            .contains("Ada leaves under clear skies.\nBen waves goodbye.")
    );
    assert!(!preview.user.contains("Ada leaves in rain."));
    assert!(!preview.user.contains("STALE MIRROR SCRIPT"));
    let mut generation = build_generate_request(&project, scene_ids[1]).unwrap();
    // Actual shared generation-admission helper; no provider call or credentials.
    attach_ai_generation_context(&mut generation, path.clone(), scene_ids[1])
        .await
        .unwrap();
    assert_eq!(
        crate::prompt_format::build_chat_prompt(&generation).user,
        preview.user
    );
    let _ = std::fs::remove_file(path);
}
