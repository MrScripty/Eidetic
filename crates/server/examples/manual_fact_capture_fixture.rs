//! Qualification fixture through unchanged public services; synthetic HTTP only.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use eidetic_server::{
    ai_generation_service, command_service, project_service, projection_service, state::AppState,
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
            name: "Saved Edit Fact QA".into(),
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
    let root = project
        .timeline
        .nodes
        .iter()
        .find(|n| n.level == StoryLevel::Premise)
        .ok_or("missing root")?
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
    let act = NodeId::new();
    let sequence = NodeId::new();
    for (id, parent, level, name) in [
        (act, root, StoryLevel::Act, "FACT RECONCILIATION"),
        (sequence, act, StoryLevel::Sequence, "Mara's umbrella"),
    ] {
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(parent),
                level,
                name: name.into(),
                start_ms: 0,
                end_ms: 180_000,
                beat_type: None,
            }),
        )
        .await?;
    }
    let ancestor_scope = std::env::var("EIDETIC_CAPTURE_SCOPE").as_deref() == Ok("ancestor-notes");
    let unrelated_act = NodeId::new();
    if ancestor_scope {
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: unrelated_act,
                parent_id: Some(root),
                level: StoryLevel::Act,
                name: "UNRELATED ACT".into(),
                start_ms: 180_000,
                end_ms: 240_000,
                beat_type: None,
            }),
        )
        .await?;
        for (id, text) in [
            (act, "  Mara conceals the witness — 雨.\n\n  "),
            (unrelated_act, "Unrelated Act: Eli guards the station."),
        ] {
            command_service::set_timeline_node_notes(
                state,
                CommandEnvelope::new(SetTimelineNodeNotesCommand {
                    node_id: id,
                    notes: text.into(),
                }),
            )
            .await?;
        }
    }
    let mut scenes = Vec::new();
    for (letter, start, end) in [
        ("A", 0, 60_000),
        ("F", 60_000, 120_000),
        ("B", 120_000, 180_000),
    ] {
        let id = NodeId::new();
        let (start, end, parent) = if ancestor_scope && letter == "F" {
            (180_000, 240_000, unrelated_act)
        } else {
            (start, end, sequence)
        };
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(parent),
                level: StoryLevel::Scene,
                name: format!("SCENE {letter}"),
                start_ms: start,
                end_ms: end,
                beat_type: None,
            }),
        )
        .await?;
        if letter == "F" {
            command_service::create_script_block(
                state,
                CommandEnvelope::new(CreateScriptBlockCommand {
                    document_id: ScriptDocumentId::new("script.document.main")?,
                    source_node_id: id,
                    expected_start_ms: start,
                    expected_end_ms: end,
                    block_kind: ScriptBlockKind::Action,
                    text: "Unrelated saved scene: Eli guards the station.\n\n".into(),
                }),
            )
            .await?;
        } else {
            command_service::set_timeline_node_notes(
                state,
                CommandEnvelope::new(SetTimelineNodeNotesCommand {
                    node_id: id,
                    notes: format!(
                        "Synthetic fixture scene {letter}: Mara carries her red umbrella."
                    ),
                }),
            )
            .await?;
        }
        scenes.push(serde_json::json!({"letter":letter,"id":id.0,"name":format!("SCENE {letter}"),"start_ms":start,"end_ms":end}));
    }
    command_service::ensure_canonical_bible_roots(
        state,
        CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
    )
    .await?;
    for (id, name, order) in [
        ("qualification.mara", "Mara", 0),
        ("qualification.eli", "Eli", 1),
    ] {
        command_service::create_bible_graph_node(state,serde_json::from_value(serde_json::json!({"id":uuid::Uuid::new_v4(),"payload":{"node_id":id,"schema_key":"character","name":name,"sort_order":order}}))?).await?;
    }
    command_service::set_bible_graph_edge(state,serde_json::from_value(serde_json::json!({"id":uuid::Uuid::new_v4(),"payload":{"edge_id":"qualification.mara.eli","from_node_id":"qualification.mara","to_node_id":"qualification.eli","edge_kind":"references","label":"Mara trusts Eli","directed":true,"sort_order":0}}))?).await?;
    field(
        state,
        "qualification.mara",
        "tagline",
        "Mara's umbrella is red.",
    )
    .await?;
    let mut generations = Vec::new();
    for index in [0, 2] {
        let id: uuid::Uuid = serde_json::from_value(scenes[index]["id"].clone())?;
        ai_generation_service::start_generation(
            state,
            ai_generation_service::AiGenerateRequest {
                node_id: id,
                story_time_ms: None,
            },
        )
        .await?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let p = projection_service::script_document_projection(
                state,
                projection_service::ScriptDocumentProjectionRequest {
                    document_id: ScriptDocumentId::new("script.document.main")?,
                },
            )
            .await?;
            if let Some(segment) = p
                .payload
                .segments
                .iter()
                .find(|s| s.segment.source_node_id.as_deref() == Some(&id.to_string()))
            {
                if !segment.blocks.is_empty() && !state.generating.lock().contains(&id) {
                    generations.push(serde_json::to_value(segment)?);
                    break;
                }
            }
            if std::time::Instant::now() > deadline {
                return Err("public synthetic generation timed out".into());
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
    // These are genuinely unconsumed: added after both generations by ordinary public writers.
    field(
        state,
        "qualification.mara",
        "motivation",
        "UNCONSUMED sibling: preserve the witness.",
    )
    .await?;
    field(
        state,
        "qualification.eli",
        "tagline",
        "UNCONSUMED Eli: brass whistle.",
    )
    .await?;
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    Ok(
        serde_json::json!({"project_path":saved["saved"],"a":scenes[0],"f":scenes[1],"b":scenes[2],"generations":generations,"ancestor":{"id":act.0,"name":"FACT RECONCILIATION"},"unrelated_act":{"id":unrelated_act.0,"name":"UNRELATED ACT"},"setup_route":"unchanged public services; two labelled synthetic HTTP generations/recaps, no real model; unconsumed fields added after generation"}),
    )
}
async fn field(
    state: &AppState,
    node: &str,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    command_service::set_bible_graph_field(
        state,
        CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new(node)?,
            part_id: BibleGraphPartId::new(format!("part.default.{node}.profile"))?,
            part_key: BibleGraphPartKey::new("profile")?,
            part_name: "Profile".into(),
            part_sort_order: 10,
            field_id: BibleGraphFieldId::new(format!("{node}.{key}"))?,
            field_key: BibleGraphFieldKey::new(key)?,
            value: Some(FieldValue::Text(value.into())),
            field_sort_order: if key == "tagline" { 20 } else { 30 },
        }),
    )
    .await?;
    Ok(())
}
