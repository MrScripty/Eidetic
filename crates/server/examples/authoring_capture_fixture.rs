//! Qualification sample: public project/notes services only; screenplay is authored in the GUI.
use eidetic_core::contracts::{CommandEnvelope, SetTimelineNodeNotesCommand};
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
            name: "Authoring Qualification".into(),
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
            expected: None,
            node_id: b.id,
            notes: "Eli waits for Mara at the station. Continue the preceding screenplay.".into(),
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
