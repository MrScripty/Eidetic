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
    if std::env::var("EIDETIC_CAPTURE_SCOPE").as_deref() == Ok("arc-description") {
        return prepare_arc_description(state).await;
    }
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
    let unrelated_sequence = NodeId::new();
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
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: unrelated_sequence,
                parent_id: Some(unrelated_act),
                level: StoryLevel::Sequence,
                name: "Unrelated station sequence".into(),
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
            (180_000, 240_000, unrelated_sequence)
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
    // Test-only public setup qualification; stop before any provider I/O.
    if std::env::var("EIDETIC_CAPTURE_SETUP_ONLY").as_deref() == Ok("1") {
        let saved = project_service::save_project(
            state,
            project_service::SaveProjectRequest { path: None },
        )
        .await?;
        return Ok(serde_json::json!({
            "setup_only": true, "provider_calls": 0, "project_path": saved["saved"],
            "a": scenes[0], "f": scenes[1], "b": scenes[2],
            "ancestor": {"id": act.0}, "unrelated_act": {"id": unrelated_act.0},
            "unrelated_sequence": {"id": unrelated_sequence.0},
            "setup_route": "ordinary public command services; real canonical hierarchy and owned Notes; no inference"
        }));
    }
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
        serde_json::json!({"project_path":saved["saved"],"a":scenes[0],"f":scenes[1],"b":scenes[2],"generations":generations,"ancestor":{"id":act.0,"name":"FACT RECONCILIATION"},"unrelated_act":{"id":unrelated_act.0,"name":"UNRELATED ACT"},"unrelated_sequence":{"id":unrelated_sequence.0},"setup_route":"unchanged public services; two labelled synthetic HTTP generations/recaps, no real model; unconsumed fields added after generation"}),
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

/// Keep the template's real tags: no tag command exists in this bounded UI.
async fn prepare_arc_description(
    state: &AppState,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let project = state
        .project
        .lock()
        .as_ref()
        .ok_or("missing template")?
        .clone();
    let arc = project
        .arcs
        .iter()
        .find(|a| a.name == "A-Plot")
        .ok_or("missing A arc")?
        .id;
    let unrelated_arc = project
        .arcs
        .iter()
        .find(|a| a.name == "C-Runner")
        .ok_or("missing C arc")?
        .id;
    let mut consumers: Vec<_> = project
        .timeline
        .nodes
        .iter()
        .filter(|n| {
            n.level == StoryLevel::Scene
                && project
                    .timeline
                    .node_arcs
                    .iter()
                    .any(|t| t.node_id == n.id && t.arc_id == arc)
        })
        .cloned()
        .collect();
    consumers.sort_by_key(|n| n.time_range.start_ms);
    let a = consumers
        .first()
        .ok_or("missing first tagged consumer")?
        .clone();
    let b = consumers
        .get(1)
        .ok_or("missing second tagged consumer")?
        .clone();
    let f = project
        .timeline
        .nodes
        .iter()
        .find(|n| {
            n.level == StoryLevel::Scene
                && project
                    .timeline
                    .node_arcs
                    .iter()
                    .any(|t| t.node_id == n.id && t.arc_id == unrelated_arc)
        })
        .ok_or("missing unrelated tagged scene")?
        .clone();
    for node in project
        .timeline
        .nodes
        .iter()
        .filter(|n| n.level == StoryLevel::Scene && ![a.id, b.id, f.id].contains(&n.id))
    {
        command_service::delete_timeline_node(
            state,
            CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: node.id }),
        )
        .await?;
    }
    for act in project.timeline.nodes.iter().filter(|n| {
        n.level == StoryLevel::Act && ![a.parent_id, b.parent_id, f.parent_id].contains(&Some(n.id))
    }) {
        command_service::delete_timeline_node(
            state,
            CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: act.id }),
        )
        .await?;
    }
    // Explicit public empty write gives the omitted Description an owned clock.
    command_service::update_story_arc(
        state,
        CommandEnvelope::new(SetStoryArcMetadataCommand {
            arc_id: arc,
            name: Some("Mara's choice".into()),
            description: Some(String::new()),
            arc_type: None,
            color: None,
        }),
    )
    .await?;
    command_service::update_story_arc(
        state,
        CommandEnvelope::new(SetStoryArcMetadataCommand {
            arc_id: unrelated_arc,
            name: Some("Unrelated direction".into()),
            description: Some(String::new()),
            arc_type: None,
            color: None,
        }),
    )
    .await?;
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
    command_service::create_script_block(
        state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main")?,
            source_node_id: f.id,
            expected_start_ms: f.time_range.start_ms,
            expected_end_ms: f.time_range.end_ms,
            block_kind: ScriptBlockKind::Action,
            text: "Unrelated saved scene: Eli guards the station.\n\n".into(),
        }),
    )
    .await?;
    for node in [&a, &b] {
        command_service::set_timeline_node_notes(
            state,
            CommandEnvelope::new(SetTimelineNodeNotesCommand {
                node_id: node.id,
                notes: "Synthetic fixture: Mara carries her red umbrella.".into(),
            }),
        )
        .await?;
    }
    let mut generations = Vec::new();
    let setup_only = std::env::var("EIDETIC_CAPTURE_SETUP_ONLY").as_deref() == Ok("1");
    if !setup_only {
        for node in [&a, &b] {
            ai_generation_service::start_generation(
                state,
                ai_generation_service::AiGenerateRequest {
                    node_id: node.id.0,
                    story_time_ms: None,
                },
            )
            .await?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                let projection = projection_service::script_document_projection(
                    state,
                    projection_service::ScriptDocumentProjectionRequest {
                        document_id: ScriptDocumentId::new("script.document.main")?,
                    },
                )
                .await?;
                if let Some(segment) =
                    projection.payload.segments.iter().find(|s| {
                        s.segment.source_node_id.as_deref() == Some(&node.id.0.to_string())
                    })
                {
                    if !segment.blocks.is_empty() && !state.generating.lock().contains(&node.id.0) {
                        generations.push(serde_json::to_value(segment)?);
                        break;
                    }
                }
                if std::time::Instant::now() > deadline {
                    return Err("public arc synthetic generation timed out".into());
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
    }
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    let scene = |node: &eidetic_core::timeline::node::StoryNode| serde_json::json!({"id":node.id.0,"name":node.name,"start_ms":node.time_range.start_ms,"end_ms":node.time_range.end_ms});
    Ok(
        serde_json::json!({"project_path":saved["saved"],"a":scene(&a),"b":scene(&b),"f":scene(&f),"arc":{"id":arc.0,"name":"Mara's choice"},"unrelated_arc":{"id":unrelated_arc.0,"name":"Unrelated direction"},"generations":generations,"setup_only":setup_only,"provider_calls":if setup_only {0}else{4},"setup_route":"ordinary public services preserve actual template Scene tags; owned empty Description write; labelled synthetic HTTP generations only"}),
    )
}
