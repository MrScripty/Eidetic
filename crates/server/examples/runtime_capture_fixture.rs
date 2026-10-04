//! Local capture data created through public project and canonical script services.
//! This example does not mock a provider, mutate SQLite directly, or run an app substitute.
use eidetic_core::contracts::{
    CommandEnvelope, ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanProvenance, SetScriptBlockCommand,
};
use eidetic_core::timeline::node::StoryLevel;
use eidetic_server::{command_service, project_service, projection_service, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new().await;
    let result = prepare(&state).await;
    state.shutdown_tasks_async().await;
    println!("{}", result?);
    Ok(())
}

async fn prepare(state: &AppState) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    const NAME: &str = "Night Shift - Capture Sample";
    project_service::create_project(
        state,
        project_service::CreateProjectRequest {
            name: NAME.into(),
            template: "single_cam".into(),
        },
    )
    .await?;
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    let scene = state
        .project
        .lock()
        .as_ref()
        .ok_or("created project missing")?
        .timeline
        .nodes
        .iter()
        .filter(|node| node.level == StoryLevel::Scene)
        .min_by_key(|node| node.time_range.start_ms)
        .ok_or("template has no scene")?
        .clone();
    let document_id = ScriptDocumentId::new("script.document.main")?;
    let block_id = ScriptBlockId::new("capture.screenplay.block")?;
    command_service::set_script_block(
        state,
        CommandEnvelope::new(SetScriptBlockCommand {
            document_id: document_id.clone(),
            document_title: NAME.into(),
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new("capture.screenplay.segment")?,
            source_node_id: Some(scene.id.0.to_string()),
            segment_start_ms: scene.time_range.start_ms,
            segment_end_ms: scene.time_range.end_ms,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: 0,
            block_id: block_id.clone(),
            block_kind: ScriptBlockKind::Action,
            text: "INT. NIGHT SHIFT CAFE - NIGHT\n\nMara sets two cups beside a folded timetable.\n\nELI\nWe still have time for the last train.".into(),
            span_provenance: ScriptSpanProvenance::Imported,
            sort_order: 0,
        }),
    )
    .await?;
    project_service::save_project(state, project_service::SaveProjectRequest { path: None })
        .await?;
    let projection = projection_service::script_document_projection(
        state,
        projection_service::ScriptDocumentProjectionRequest { document_id },
    )
    .await?;
    let block = projection
        .payload
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == block_id)
        .ok_or("canonical fixture block missing")?;
    Ok(serde_json::json!({
        "project_name": NAME,
        "project_path": saved["saved"],
        "block_id": block_id,
        "initial_revision_event_id": block.revision_event_id,
        "scene_name": scene.name,
        "text": block.block.text,
    }))
}
