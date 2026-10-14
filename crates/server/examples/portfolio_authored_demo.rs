//! Original authored demo. Calls public backend commands; performs no inference.
//! Run with a fresh XDG_DATA_HOME to keep personal projects separate.
use eidetic_core::{
    contracts::*,
    timeline::node::{NodeId, StoryLevel},
};
use eidetic_server::{command_service as commands, project_service as projects, state::AppState};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("eidetic/projects/Signal_at_Low_Tide/project.db");
    if path.exists() {
        return Err(format!(
            "Demo project already exists at {}; use a fresh XDG_DATA_HOME",
            path.display()
        )
        .into());
    }
    let state = AppState::new().await;
    let result = prepare(&state).await;
    state.shutdown_tasks_async().await;
    println!("{}", result?);
    Ok(())
}

async fn prepare(state: &AppState) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    projects::create_project(
        state,
        projects::CreateProjectRequest {
            name: "Signal at Low Tide".into(),
            template: "single_cam".into(),
        },
    )
    .await?;
    projects::update_project(state, projects::UpdateProjectRequest {
        name: None,
        premise: Some("When a decommissioned lighthouse answers her father's call sign, a radio technician must find the sender before the island's last night of light.".into()),
    })?;
    projects::save_project(state, projects::SaveProjectRequest { path: None }).await?;
    let project = state.project.lock().as_ref().unwrap().clone();
    let root = project
        .timeline
        .nodes
        .iter()
        .find(|n| n.level == StoryLevel::Premise)
        .unwrap()
        .id;
    for act in project
        .timeline
        .nodes
        .iter()
        .filter(|n| n.level == StoryLevel::Act)
    {
        commands::delete_timeline_node(
            state,
            CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: act.id }),
        )
        .await?;
    }
    let acts = [
        "ACT I | Reception",
        "ACT II | Echoes",
        "ACT III | Transmission",
    ];
    let scene_names = [
        "01 | A signal at dusk",
        "02 | The harbor ledger",
        "03 | A room of echoes",
        "04 | The missing page",
        "05 | Light in the fog",
        "06 | An answer at dawn",
    ];
    let scene_notes = [
        "Mara discovers a light in the abandoned lighthouse. Ivo insists the power is gone. A radio pulse carries her father's call sign.",
        "The harbor ledger places the last service boat at Northbreak on the night her father disappeared. Ivo has torn out one page.",
        "Inside the lighthouse, Mara finds a warm relay and a reel-to-reel recorder. The voice on the tape is not the voice on the radio.",
        "Ivo admits he kept the last log entry to protect Mara. She chooses to hear the whole story rather than another version of it.",
        "A storm cuts the road. Mara bridges the old relay by hand while Ivo climbs to the lamp room. They send the call together.",
        "A fishing boat answers. Her father once saved its crew. At dawn, Mara leaves the receiver on and watches the island wake.",
    ];
    let mut scenes = Vec::new();
    for (i, name) in acts.iter().enumerate() {
        let act = NodeId(uuid::Uuid::new_v4());
        let sequence = NodeId(uuid::Uuid::new_v4());
        let start = (i as u64) * 440_000;
        for (id, parent, level, label, lo, hi) in [
            (act, root, StoryLevel::Act, *name, start, start + 440_000),
            (
                sequence,
                act,
                StoryLevel::Sequence,
                ["The shore", "The keeper's story", "The last light"][i],
                start,
                start + 440_000,
            ),
        ] {
            commands::create_timeline_node_from_core_command(
                state,
                CommandEnvelope::new(CreateTimelineNodeCommand {
                    node_id: id,
                    parent_id: Some(parent),
                    level,
                    name: label.into(),
                    start_ms: lo,
                    end_ms: hi,
                    beat_type: None,
                }),
            )
            .await?;
        }
        for j in 0..2 {
            let index = i * 2 + j;
            let id = NodeId(uuid::Uuid::new_v4());
            let lo = start + (j as u64) * 220_000;
            let hi = lo + 220_000;
            commands::create_timeline_node_from_core_command(
                state,
                CommandEnvelope::new(CreateTimelineNodeCommand {
                    node_id: id,
                    parent_id: Some(sequence),
                    level: StoryLevel::Scene,
                    name: scene_names[index].into(),
                    start_ms: lo,
                    end_ms: hi,
                    beat_type: None,
                }),
            )
            .await?;
            commands::set_timeline_node_notes(
                state,
                CommandEnvelope::new(SetTimelineNodeNotesCommand {
                    node_id: id,
                    notes: scene_notes[index].into(),
                    expected: None,
                }),
            )
            .await?;
            scenes.push((id, lo, hi));
        }
    }
    let first_blocks = [
        (
            ScriptBlockKind::SceneHeading,
            "EXT. NORTHBREAK HARBOR - DUSK",
        ),
        (
            ScriptBlockKind::Action,
            "The last ferry has gone. A ribbon of fog gathers beyond the breakwater.\n\nMARA VALE, 32, crouches beside a marine receiver. Salt on the dial. A strip of blue tape over the broken handle.",
        ),
        (ScriptBlockKind::Character, "IVO"),
        (
            ScriptBlockKind::Dialogue,
            "They cut the power to that lighthouse six years ago.",
        ),
        (
            ScriptBlockKind::Action,
            "A lamp turns once in the distance. Then the receiver clicks.",
        ),
        (ScriptBlockKind::Character, "MARA"),
        (
            ScriptBlockKind::Dialogue,
            "Then someone forgot to tell the light.",
        ),
        (
            ScriptBlockKind::Action,
            "She adjusts the dial. Three short pulses. Her father's call sign.\n\nIvo stops winding the rope.",
        ),
    ];
    for (kind, text) in first_blocks {
        commands::create_script_block(
            state,
            CommandEnvelope::new(CreateScriptBlockCommand {
                document_id: ScriptDocumentId::new("script.document.main")?,
                source_node_id: scenes[0].0,
                expected_start_ms: scenes[0].1,
                expected_end_ms: scenes[0].2,
                block_kind: kind,
                text: text.into(),
            }),
        )
        .await?;
    }
    for (index, heading) in [
        "INT. HARBOR OFFICE - NIGHT",
        "INT. LIGHTHOUSE CONTROL ROOM - NIGHT",
        "INT. LAMP ROOM - NIGHT",
        "EXT. NORTHBREAK ROAD - PRE-DAWN",
        "EXT. NORTHBREAK HARBOR - DAWN",
    ]
    .iter()
    .enumerate()
    {
        let scene = scenes[index + 1];
        for (kind, text) in [
            (ScriptBlockKind::SceneHeading, *heading),
            (ScriptBlockKind::Action, scene_notes[index + 1]),
        ] {
            commands::create_script_block(
                state,
                CommandEnvelope::new(CreateScriptBlockCommand {
                    document_id: ScriptDocumentId::new("script.document.main")?,
                    source_node_id: scene.0,
                    expected_start_ms: scene.1,
                    expected_end_ms: scene.2,
                    block_kind: kind,
                    text: text.into(),
                }),
            )
            .await?;
        }
    }
    for (index, (id, schema, name, text, fields)) in [
        ("demo.mara", "character", "Mara Vale", "A marine radio technician returning to an island she thought she had left behind.", vec![("profile", "summary", "Repairs the island's radios; avoids the story of her father's disappearance."), ("profile", "tagline", "She can repair anything except a goodbye."), ("profile", "motivation", "Find the sender before the lighthouse is demolished."), ("appearance", "costume", "Salt-worn blue jacket; brass whistle.")]),
        ("demo.ivo", "character", "Ivo Rook", "Harbor keeper. His silence has protected the wrong person for too long.", vec![("profile", "summary", "The keeper of Northbreak's harbor ledger and its unfinished stories."), ("profile", "tagline", "The island's memory has a keeper.")]),
        ("demo.lighthouse", "location", "Northbreak Lighthouse", "The old lighthouse is scheduled for demolition at dawn.", vec![("environment", "description", "A dark lamp room above a warm relay cabinet. Salt, brass and six years of dust."), ("environment", "weather", "Sea fog. A spring storm gathering offshore.")]),
        ("demo.harbor", "location", "Northbreak Harbor", "The harbor is the island's threshold: every arrival carries a story.", vec![("environment", "description", "A stone breakwater, a closed ferry office and the last working radio mast.")]),
        ("demo.receiver", "prop", "Marine Receiver", "An old coastguard receiver with a broken handle and a strip of blue tape.", vec![("identity", "description", "Mara's father's receiver. Three short pulses break through its static."), ("identity", "significance", "The call sign connects an unfinished goodbye to a living witness.")]),
    ].into_iter().enumerate() {
        commands::create_bible_graph_node(state, serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"payload":{
            "node_id":id,"schema_key":schema,"name":name,"sort_order":index
        }}))?).await?;
        commands::set_bible_graph_node_text(state, CommandEnvelope::new(SetBibleGraphNodeTextCommand {
            node_id: BibleGraphNodeId::new(id)?, text: text.into(),
        })).await?;
        for (order, (part, field, value)) in fields.into_iter().enumerate() {
            commands::set_bible_graph_field(state, CommandEnvelope::new(SetBibleGraphFieldCommand {
                node_id: BibleGraphNodeId::new(id)?, part_id: BibleGraphPartId::new(format!("part.default.{id}.{part}"))?, part_key: BibleGraphPartKey::new(part)?, part_name: match part { "profile" => "Profile", "appearance" => "Appearance", "environment" => "Environment", "identity" => "Identity", _ => part }.into(), part_sort_order: if part == "appearance" {20} else {10},
                field_id: BibleGraphFieldId::new(format!("field.default.{id}.{part}.{field}"))?, field_key: BibleGraphFieldKey::new(field)?, value: Some(FieldValue::Text(value.into())), field_sort_order: ((order as u32) + 1) * 10,
            })).await?;
        }
    }
    for (index, (from, to, label)) in [
        ("demo.mara", "demo.ivo", "Questions the keeper"),
        ("demo.mara", "demo.receiver", "Repairs and carries"),
        ("demo.receiver", "demo.lighthouse", "Receives its signal"),
        ("demo.ivo", "demo.harbor", "Keeps its ledger"),
        ("demo.lighthouse", "demo.harbor", "Guides the crossing"),
    ]
    .into_iter()
    .enumerate()
    {
        commands::set_bible_graph_edge(state, serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"payload":{
            "edge_id":format!("demo.edge.{index}"),"from_node_id":from,"to_node_id":to,"edge_kind":"references","label":label,"directed":true,"sort_order":index
        }}))?).await?;
    }
    let saved = projects::save_project(state, projects::SaveProjectRequest { path: None }).await?;
    Ok(
        json!({"project_path":saved["saved"],"project_name":"Signal at Low Tide","authorship":"capture-only original authored fiction; public backend commands; no generation or inference", "scene_count":scenes.len(), "first_scene_id":scenes[0].0.0}),
    )
}
