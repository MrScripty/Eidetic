//! Fresh QA project and four supporting scene blocks through public services.
//! A is empty and B ungenerated: native writing, generation, edits, Bible Save,
//! placement, proposal review and acceptance occur after application launch.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
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
            name: "Last Train Authoring QA".into(),
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
        .find(|n| n.level == StoryLevel::Premise)
        .ok_or("root missing")?
        .id;
    for act in project
        .timeline
        .nodes
        .iter()
        .filter(|n| n.level == StoryLevel::Act)
    {
        command_service::delete_timeline_node(
            state,
            CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: act.id }),
        )
        .await?;
    }
    let act = NodeId(uuid::Uuid::new_v4());
    command_service::create_timeline_node_from_core_command(
        state,
        CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: act,
            parent_id: Some(root),
            level: StoryLevel::Act,
            name: "LAST TRAIN".into(),
            start_ms: 0,
            end_ms: 600_000,
            beat_type: None,
        }),
    )
    .await?;
    let sequence = NodeId(uuid::Uuid::new_v4());
    command_service::create_timeline_node_from_core_command(
        state,
        CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: sequence,
            parent_id: Some(act),
            level: StoryLevel::Sequence,
            name: "A lost ticket and the last connection".into(),
            start_ms: 0,
            end_ms: 600_000,
            beat_type: None,
        }),
    )
    .await?;
    let mut scenes = Vec::new();
    for (name, start, end) in [
        ("A", 60_000, 120_000),
        ("F", 120_000, 150_000),
        ("B", 240_000, 300_000),
        ("C", 360_000, 390_000),
        ("D", 480_000, 510_000),
        ("E", 540_000, 570_000),
    ] {
        let id = NodeId(uuid::Uuid::new_v4());
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(sequence),
                level: StoryLevel::Scene,
                name: format!("SCENE {name}"),
                start_ms: start,
                end_ms: end,
                beat_type: None,
            }),
        )
        .await?;
        if name != "A" && name != "B" {
            command_service::create_script_block(
                state,
                CommandEnvelope::new(CreateScriptBlockCommand {
                    document_id: ScriptDocumentId::new("script.document.main")?,
                    source_node_id: id,
                    expected_start_ms: start,
                    expected_end_ms: end,
                    block_kind: ScriptBlockKind::Action,
                    text: match name {
                        "F" => "EXT. PLATFORM - NIGHT\n\nEli keeps the gate open.\n\n",
                        "C" => {
                            "INT. TICKET OFFICE - NIGHT\n\nThe departure board reads midnight.\n\n"
                        }
                        "D" => "EXT. SIGNAL BOX - NIGHT\n\nThe green signal holds.\n\n",
                        "E" => "INT. LOST PROPERTY - NIGHT\n\nMara finds the missing ticket.\n\n",
                        _ => unreachable!(),
                    }
                    .into(),
                }),
            )
            .await?;
        }
        scenes.push(serde_json::json!({"letter": name, "id": id.0, "name": format!("SCENE {name}"), "start_ms": start, "end_ms": end}));
    }
    let b = NodeId(serde_json::from_value(scenes[2]["id"].clone())?);
    command_service::set_timeline_node_notes(
        state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id: b,
            notes: "Mara waits for Eli at the station. Consume only supplied screenplay evidence."
                .into(),
        }),
    )
    .await?;
    let create = serde_json::from_value(
        serde_json::json!({"id": uuid::Uuid::new_v4(), "payload": {
            "node_id": "qualification.mara", "schema_key": "character", "name": "Mara", "sort_order": 0
        }}),
    )?;
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
            value: Some(FieldValue::Text("Mara's umbrella is blue.".into())),
            field_sort_order: 20,
        }),
    )
    .await?;
    // Public timeline creation owns canonical rows. Normal save/open refreshes
    // the legacy project mirror that generation completion still reads.
    let prepared =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    project_service::load_project(
        state,
        project_service::LoadProjectRequest {
            path: prepared["saved"]
                .as_str()
                .ok_or("prepared path missing")?
                .into(),
        },
    )
    .await?;
    if state
        .project
        .lock()
        .as_ref()
        .is_none_or(|p| p.timeline.node(b).is_err())
    {
        return Err("prepared B absent from refreshed project mirror".into());
    }
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    Ok(
        serde_json::json!({"project_path": saved["saved"], "scenes": scenes, "a": scenes[0], "f": scenes[1], "b": scenes[2], "c": scenes[3], "e": scenes[5],
        "before_order": ["A","F","B","C","D","E"], "after_order": ["A","F","E","B","C","D"],
        "reorder_route": "native exact placement form after launch", "setup_route": "public services only; A empty and B ungenerated" }),
    )
}
