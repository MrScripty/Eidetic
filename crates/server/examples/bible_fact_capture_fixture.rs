//! Bible propagation qualification: public setup services; actual fact change and screenplay review use the GUI.
use eidetic_core::contracts::*;
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
            name: "Bible Fact Qualification".into(),
            template: "single_cam".into(),
        },
    )
    .await?;
    project_service::save_project(state, project_service::SaveProjectRequest { path: None })
        .await?;
    let mut scenes = state
        .project
        .lock()
        .as_ref()
        .ok_or("project missing")?
        .timeline
        .nodes
        .iter()
        .filter(|node| node.level == StoryLevel::Scene)
        .cloned()
        .collect::<Vec<_>>();
    scenes.sort_by_key(|node| node.time_range.start_ms);
    let [a, b, ..] = scenes.as_slice() else {
        return Err("two scenes required".into());
    };
    command_service::set_timeline_node_notes(
        state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id: b.id,
            notes: "Eli waits for Mara at the station. Continue the preceding screenplay.".into(),
        }),
    )
    .await?;
    let create = serde_json::from_value(serde_json::json!({
        "id": uuid::Uuid::new_v4(), "payload": {
            "node_id": "qualification.mara", "schema_key": "character", "name": "Mara", "sort_order": 0
        }
    }))?;
    command_service::create_bible_graph_node(state, create).await?;
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
            value: Some(FieldValue::Text("Mara's umbrella is red.".into())),
            field_sort_order: 20,
        }),
    )
    .await?;
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    Ok(
        serde_json::json!({"project_path": saved["saved"], "a": {"id": a.id.0, "name": a.name,
        "start_ms": a.time_range.start_ms, "end_ms": a.time_range.end_ms},
        "b": {"id": b.id.0, "name": b.name}}),
    )
}
