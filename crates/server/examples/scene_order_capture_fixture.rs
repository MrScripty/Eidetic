//! Native qualification setup through public services. Reorder occurs here;
//! manual screenplay edit, preview and acceptance occur in the real GUI.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use eidetic_server::{
    ai_generation_service, command_service, project_service, projection_service,
    state::{AiConfig, AppState},
};

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
            name: "Scene Order Qualification".into(),
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
            name: "Scene order".into(),
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
            name: "Continuity window".into(),
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
        if name != "B" {
            command_service::create_script_block(
                state,
                CommandEnvelope::new(CreateScriptBlockCommand {
                    document_id: ScriptDocumentId::new("script.document.main")?,
                    source_node_id: id,
                    expected_start_ms: start,
                    expected_end_ms: end,
                    block_kind: ScriptBlockKind::Action,
                    text: format!("Exact authored scene {name}.\n\n"),
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
            notes: "B waits at the station; consume only supplied screenplay evidence.".into(),
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
    *state.ai_config.lock() = AiConfig {
        base_url: "http://127.0.0.1:18080/v1".into(),
        model: "authoring-http-fixture".into(),
        ..AiConfig::default()
    };
    ai_generation_service::start_generation(
        state,
        ai_generation_service::AiGenerateRequest {
            node_id: b.0,
            story_time_ms: None,
        },
    )
    .await?;
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        while state.generating.lock().contains(&b.0) {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
    .await?;
    let before = projection_service::script_document_projection(
        state,
        projection_service::ScriptDocumentProjectionRequest {
            document_id: ScriptDocumentId::new("script.document.main")?,
        },
    )
    .await?;
    let generated = before
        .payload
        .segments
        .iter()
        .find(|s| s.segment.source_node_id.as_deref() == Some(b.0.to_string().as_str()))
        .ok_or("generated B missing")?;
    if generated.blocks.first().map(|b| b.block.text.as_str())
        != Some("Synthetic generation: B waits at the station.\n\n")
        || generated.impact.as_ref().is_none_or(|i| i.needs_review)
    {
        return Err("initial generation failed or already needs review".into());
    }
    let e = NodeId(serde_json::from_value(scenes[5]["id"].clone())?);
    // Actual six-scene order changes from A,F,B,C,D,E to A,F,E,B,C,D.
    command_service::set_timeline_node_range(
        state,
        CommandEnvelope::new(SetTimelineNodeRangeCommand {
            node_id: e,
            start_ms: 180_000,
            end_ms: 210_000,
        }),
    )
    .await?;
    let after = projection_service::script_document_projection(
        state,
        projection_service::ScriptDocumentProjectionRequest {
            document_id: ScriptDocumentId::new("script.document.main")?,
        },
    )
    .await?;
    let reviewed = after
        .payload
        .segments
        .iter()
        .find(|s| s.segment.id == generated.segment.id)
        .ok_or("B missing after move")?;
    if reviewed.blocks != generated.blocks
        || reviewed.impact.as_ref().is_none_or(|i| {
            !i.causes.iter().any(|c| {
                c.reason == ScriptImpactReason::ContextChanged
                    && c.input_excerpt.as_deref() == Some("Entered: SCENE E. Left: SCENE A.")
            })
        })
    {
        return Err("reorder did not preserve B and derive exact membership cause".into());
    }
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    Ok(
        serde_json::json!({"project_path": saved["saved"], "scenes": scenes, "b": scenes[2],
        "before_order": ["A","F","B","C","D","E"], "after_order": ["A","F","E","B","C","D"],
        "reorder_route": "public native set_timeline_node_range before GUI launch", "target_after_reorder": reviewed }),
    )
}
