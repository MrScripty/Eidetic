//! Native qualification setup only. The new scene is created/selected/authored
//! and generated later through actual GUI input, without a save/reopen workaround.
use eidetic_core::{
    contracts::*,
    timeline::node::{NodeId, StoryLevel},
};
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
            name: "Canonical Scene Qualification".into(),
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
        .ok_or("project missing")?
        .clone();
    let root = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.level == StoryLevel::Premise)
        .ok_or("root missing")?
        .id;
    for act in project
        .timeline
        .nodes
        .iter()
        .filter(|node| node.level == StoryLevel::Act)
    {
        command_service::delete_timeline_node(
            state,
            CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: act.id }),
        )
        .await?;
    }
    let act = NodeId(uuid::Uuid::new_v4());
    let sequence = NodeId(uuid::Uuid::new_v4());
    let a = NodeId(uuid::Uuid::new_v4());
    for (id, parent, level, name, start, end) in [
        (act, root, StoryLevel::Act, "Canonical authoring", 0, 600000),
        (
            sequence,
            act,
            StoryLevel::Sequence,
            "Canonical sequence",
            0,
            600000,
        ),
        (a, sequence, StoryLevel::Scene, "SCENE A", 60000, 120000),
    ] {
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(parent),
                level,
                name: name.into(),
                start_ms: start,
                end_ms: end,
                beat_type: None,
            }),
        )
        .await?;
    }
    command_service::create_script_block(
        state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main")?,
            source_node_id: a,
            expected_start_ms: 60000,
            expected_end_ms: 120000,
            block_kind: ScriptBlockKind::Action,
            text: "Exact existing authored scene A.\n\n".into(),
        }),
    )
    .await?;
    command_service::create_bible_graph_node(
        state,
        serde_json::from_value(serde_json::json!({"id":uuid::Uuid::new_v4(),"payload":{
            "node_id":"qualification.mara","schema_key":"character","name":"Mara","sort_order":0
        }}))?,
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
    project_service::save_project(state, project_service::SaveProjectRequest { path: None })
        .await?;
    let path = state
        .project_database
        .active_path()
        .ok_or("prepared project path missing")?;
    Ok(
        serde_json::json!({"path":path,"sequence":{"id":sequence.0,"name":"Canonical sequence"},
        "a":{"id":a.0,"name":"SCENE A"},"setup_route":"public services prepare existing A/parent/Bible only; new scene must be created in GUI"}),
    )
}
