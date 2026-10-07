//! Public-service QA setup. Native controls own generation, exact arc editing,
//! manual screenplay Save, preview, stale refusal and explicit acceptance.
use eidetic_core::contracts::*;
use eidetic_core::story::arc::ArcType;
use eidetic_core::timeline::node::StoryLevel;
use eidetic_server::{command_service, project_service, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new().await;
    let result = prepare(&state).await;
    state.shutdown_tasks_async().await;
    println!("{}", result?);
    Ok(())
}

async fn prepare(state: &AppState) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    project_service::create_project(
        state,
        project_service::CreateProjectRequest {
            name: "Witness Arc Memory QA".into(),
            template: "multi_cam".into(),
        },
    )
    .await?;
    project_service::save_project(state, project_service::SaveProjectRequest { path: None })
        .await?;
    let project = state
        .project
        .lock()
        .as_ref()
        .ok_or("missing project")?
        .clone();
    let arc = project
        .arcs
        .iter()
        .find(|arc| arc.arc_type == ArcType::APlot)
        .ok_or("missing A arc")?;
    let mut scenes: Vec<_> = project
        .timeline
        .nodes
        .iter()
        .filter(|node| node.level == StoryLevel::Scene)
        .collect();
    scenes.sort_by_key(|node| node.time_range.start_ms);
    let b = scenes
        .iter()
        .find(|node| project.timeline.arcs_for_node(node.id).contains(&arc.id))
        .ok_or("missing tagged scene")?;
    let f = scenes
        .iter()
        .find(|node| !project.timeline.arcs_for_node(node.id).contains(&arc.id))
        .ok_or("missing unrelated scene")?;
    command_service::update_story_arc(
        state,
        CommandEnvelope::new(SetStoryArcMetadataCommand {
            arc_id: arc.id,
            name: Some("Witness".into()),
            description: Some("Mara conceals the witness's identity.".into()),
            arc_type: None,
            color: None,
        }),
    )
    .await?;
    command_service::set_timeline_node_notes(
        state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id: b.id,
            notes: "Mara waits for Eli. Follow the supplied Witness arc and authored screenplay."
                .into(),
        }),
    )
    .await?;
    command_service::create_script_block(
        state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main")?,
            source_node_id: f.id,
            expected_start_ms: f.time_range.start_ms,
            expected_end_ms: f.time_range.end_ms,
            block_kind: ScriptBlockKind::Action,
            text: "EXT. PLATFORM - NIGHT\n\nEli keeps the gate open.\n\n".into(),
        }),
    )
    .await?;
    command_service::create_bible_graph_node(
        state,
        serde_json::from_value(serde_json::json!({
            "id": uuid::Uuid::new_v4(), "payload": {"node_id": "qualification.mara",
            "schema_key": "character", "name": "Mara", "sort_order": 0}
        }))?,
    )
    .await?;
    command_service::set_bible_graph_field(
        state,
        CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new("qualification.mara")?,
            part_id: BibleGraphPartId::new("part.default.qualification.mara.profile")?,
            part_key: BibleGraphPartKey::new("profile")?,
            part_name: "Profile".into(),
            part_sort_order: 10,
            field_id: BibleGraphFieldId::new("qualification.mara.tagline")?,
            field_key: BibleGraphFieldKey::new("tagline")?,
            value: Some(FieldValue::Text("Mara's umbrella is blue.".into())),
            field_sort_order: 20,
        }),
    )
    .await?;
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    Ok(
        serde_json::json!({"project_path": saved["saved"], "arc": {"id": arc.id.0, "name": "Witness"},
        "b": {"id": b.id.0, "name": b.name, "start_ms": b.time_range.start_ms, "end_ms": b.time_range.end_ms},
        "f": {"id": f.id.0, "name": f.name, "start_ms": f.time_range.start_ms, "end_ms": f.time_range.end_ms},
        "setup_route": "public project/template, arc metadata, notes, manual block and Bible services; no direct DB writes"}),
    )
}
