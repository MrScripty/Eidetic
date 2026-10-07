//! Native QA setup exclusively through existing public project/command/read services.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use eidetic_server::{bible_recall_service, command_service, project_service, state::AppState};

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
            name: "Related Facts Recall QA".into(),
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
    let act = NodeId::new();
    let sequence = NodeId::new();
    for (id, parent_id, level, name) in [
        (act, root, StoryLevel::Act, "RELATED FACTS"),
        (
            sequence,
            act,
            StoryLevel::Sequence,
            "Mara and the beach house",
        ),
    ] {
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(parent_id),
                level,
                name: name.into(),
                start_ms: 0,
                end_ms: 240_000,
                beat_type: None,
            }),
        )
        .await?;
    }
    let mut scenes = Vec::new();
    for (letter, start, end, text) in [
        (
            "A",
            0,
            60_000,
            Some("INT. CAFE - NIGHT\n\nMara folds her blue umbrella.\n\n"),
        ),
        (
            "F",
            60_000,
            120_000,
            Some("EXT. PLATFORM - NIGHT\n\nEli keeps the gate open.\n\n"),
        ),
        ("B", 120_000, 180_000, None),
    ] {
        let id = NodeId::new();
        command_service::create_timeline_node_from_core_command(
            state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(sequence),
                level: StoryLevel::Scene,
                name: format!("SCENE {letter}"),
                start_ms: start,
                end_ms: end,
                beat_type: None,
            }),
        )
        .await?;
        if let Some(text) = text {
            command_service::create_script_block(
                state,
                CommandEnvelope::new(CreateScriptBlockCommand {
                    document_id: ScriptDocumentId::new("script.document.main")?,
                    source_node_id: id,
                    expected_start_ms: start,
                    expected_end_ms: end,
                    block_kind: ScriptBlockKind::Action,
                    text: text.into(),
                }),
            )
            .await?;
        } else {
            command_service::set_timeline_node_notes(state, CommandEnvelope::new(SetTimelineNodeNotesCommand {
                node_id: id, notes: "Mara waits for Eli at the station. Consume only supplied screenplay evidence.".into(),
            })).await?;
        }
        scenes.push(serde_json::json!({"letter":letter,"id":id.0,"name":format!("SCENE {letter}"),"start_ms":start,"end_ms":end}));
    }
    command_service::ensure_canonical_bible_roots(
        state,
        CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
    )
    .await?;
    create(state, "qualification.mara", "character", "Mara", 0).await?;
    for index in 0..205 {
        create(
            state,
            &format!("qualification.filler.{index:03}"),
            "prop",
            &format!("Unrelated prop {index:03}"),
            index + 1,
        )
        .await?;
    }
    create(
        state,
        "qualification.beach-house",
        "location",
        "Beach House",
        10_000,
    )
    .await?;
    command_service::set_bible_graph_edge(state, serde_json::from_value(serde_json::json!({
        "id":uuid::Uuid::new_v4(),"payload":{"edge_id":"qualification.mara.home","from_node_id":"qualification.mara",
        "to_node_id":"qualification.beach-house","edge_kind":"located_in","label":"Mara's home","directed":true,"sort_order":0}
    }))?).await?;
    field(
        state,
        "qualification.mara",
        "profile",
        "tagline",
        "Mara's umbrella is blue.",
    )
    .await?;
    field(
        state,
        "qualification.beach-house",
        "environment",
        "weather",
        "Dry",
    )
    .await?;
    for (id, at, key, value) in [
        ("qualification.house.opening", 1000, "weather", "Rain"),
        (
            "qualification.house.future",
            2000,
            "description",
            "Flooded road",
        ),
    ] {
        command_service::set_bible_graph_snapshot_field(state, serde_json::from_value(serde_json::json!({
            "id":uuid::Uuid::new_v4(),"payload":{"snapshot_id":id,"node_id":"qualification.beach-house",
            "at_ms":at,"label":if at==1000 {"Opening"} else {"Future"},"field_id":format!("{id}.{key}"),
            "part_key":"environment","part_name":"Environment","field_key":key,
            "value":{"type":"text","value":value},"field_sort_order":0,"snapshot_sort_order":0}
        }))?).await?;
    }
    create(
        state,
        "qualification.keeper",
        "character",
        "The Keeper",
        10_001,
    )
    .await?;
    field(
        state,
        "qualification.keeper",
        "profile",
        "tagline",
        "SELECTED dry roof key — 雨.",
    )
    .await?;
    field(
        state,
        "qualification.keeper",
        "profile",
        "motivation",
        "UNSELECTED brass key in cellar.",
    )
    .await?;
    command_service::set_bible_graph_edge(state, serde_json::from_value(serde_json::json!({
        "id":uuid::Uuid::new_v4(),"payload":{"edge_id":"qualification.mara.keeper","from_node_id":"qualification.mara",
        "to_node_id":"qualification.keeper","edge_kind":{"custom":"trusted contact"},"label":"Mara trusts the Keeper","directed":true,"sort_order":1}
    }))?).await?;
    let recall = bible_recall_service::recall(
        state,
        BibleRecallRequest {
            anchor_node_id: BibleGraphNodeId::new("qualification.mara")?,
            story_time_ms: Some(1000),
            direction: BibleRecallDirection::Both,
            edge_kinds: vec![],
            neighbor_limit: 8,
        },
    )
    .await?;
    if recall.payload.nodes.len() != 3 || recall.payload.nodes[1].name != "Beach House" {
        return Err("public recall did not select the linked entity beyond the prefix".into());
    }
    let prepared =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    project_service::load_project(
        state,
        project_service::LoadProjectRequest {
            path: prepared["saved"]
                .as_str()
                .ok_or("saved path missing")?
                .into(),
        },
    )
    .await?;
    let saved =
        project_service::save_project(state, project_service::SaveProjectRequest { path: None })
            .await?;
    Ok(
        serde_json::json!({"project_path":saved["saved"],"scenes":scenes,"a":scenes[0],"f":scenes[1],"b":scenes[2],
        "setup_route":"public services only; exact manual A/F supporting text, native B generation/edit after launch",
        "mara":"qualification.mara","neighbor":"qualification.beach-house","filler_count":205,"selected_peer":"qualification.keeper",
        "public_recall":recall,"model_execution_in_setup":"none"}),
    )
}
async fn create(
    state: &AppState,
    id: &str,
    schema: &str,
    name: &str,
    order: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    command_service::create_bible_graph_node(state,serde_json::from_value(serde_json::json!({
        "id":uuid::Uuid::new_v4(),"payload":{"node_id":id,"schema_key":schema,"name":name,"sort_order":order}
    }))?).await?;
    Ok(())
}
async fn field(
    state: &AppState,
    id: &str,
    part: &str,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    command_service::set_bible_graph_field(
        state,
        CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new(id)?,
            part_id: BibleGraphPartId::new(format!("part.default.{id}.{part}"))?,
            part_key: BibleGraphPartKey::new(part)?,
            part_name: if part == "profile" {
                "Profile".into()
            } else {
                "Environment".into()
            },
            part_sort_order: 10,
            field_id: BibleGraphFieldId::new(format!("{id}.{key}"))?,
            field_key: BibleGraphFieldKey::new(key)?,
            value: Some(FieldValue::Text(value.into())),
            field_sort_order: 20,
        }),
    )
    .await?;
    Ok(())
}
