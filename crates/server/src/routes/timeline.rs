use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel, StoryNode};
use eidetic_core::timeline::relationship::{Relationship, RelationshipId, RelationshipType};
use eidetic_core::timeline::timing::TimeRange;
use eidetic_core::timeline::track::TrackId;

use crate::error::{json_value, ApiError, ApiJson};
use crate::state::{AppState, ServerEvent};
use crate::validation;
use crate::ydoc::{ContentField, DocCommand};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline", get(get_timeline))
        .route("/timeline/nodes/{id}/apply-children", post(apply_children))
        .route("/timeline/nodes/{id}/children", get(get_children))
        .route("/timeline/tracks", post(create_track))
        .route("/timeline/tracks/{id}", put(update_track))
        .route("/timeline/tracks/{id}", delete(delete_track))
        .route("/timeline/node-arcs", post(tag_node_with_arc))
        .route(
            "/timeline/node-arcs/{node_id}/{arc_id}",
            delete(untag_node_from_arc),
        )
        .route("/timeline/relationships", post(create_relationship))
        .route(
            "/timeline/relationships/{id}",
            delete(delete_relationship),
        )
        .route("/timeline/gaps", get(get_gaps))
}

// ─── Timeline ──────────────────────────────────────────────────────

async fn get_timeline(State(state): State<AppState>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => json_value(&p.timeline),
        None => Err(ApiError::no_project()),
    }
}

// ─── Node Queries And Planning ─────────────────────────────────────

async fn get_children(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let children = p.timeline.children_of(NodeId(id));
            json_value(&children)
        }
        None => Err(ApiError::no_project()),
    }
}

// ─── Apply Children (from AI proposals) ────────────────────────────

#[derive(Deserialize)]
struct ApplyChildrenRequest {
    children: Vec<ApplyChildEntry>,
}

#[derive(Deserialize)]
struct ApplyChildEntry {
    name: String,
    beat_type: Option<String>,
    outline: String,
    weight: f32,
    #[serde(default)]
    characters: Vec<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    props: Vec<String>,
}

async fn apply_children(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ApplyChildrenRequest>,
) -> ApiJson {
    use eidetic_core::story::arc::Color;
    use eidetic_core::story::bible::{Entity, EntityCategory, EntityDetails};
    use eidetic_core::timeline::node::ContentStatus;

    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let parent_id = NodeId(id);

    // Get parent node info.
    let (parent_range, parent_level) = match project.timeline.node(parent_id) {
        Ok(n) => (n.time_range, n.level),
        Err(e) => return Err(ApiError::bad_request(e.to_string())),
    };

    let child_level = match parent_level.child_level() {
        Some(l) => l,
        None => return Err(ApiError::bad_request("this node level cannot have children")),
    };

    // Clear existing children of this parent.
    if let Err(e) = project.timeline.clear_children_of(parent_id) {
        return Err(ApiError::bad_request(e.to_string()));
    }

    // Calculate total weight.
    let total_weight: f32 = body.children.iter().map(|c| c.weight.max(0.1)).sum();
    let parent_duration = parent_range.end_ms - parent_range.start_ms;

    // Distribute time proportionally.
    let mut cursor = parent_range.start_ms;
    let mut created_nodes = Vec::new();
    let mut node_entities: Vec<(NodeId, Vec<String>, Option<String>, Vec<String>)> = Vec::new();

    // Copy arc tags from parent to children.
    let parent_arc_ids = project.timeline.arcs_for_node(parent_id);

    for (i, entry) in body.children.iter().enumerate() {
        validation::validate_name(&entry.name, "child node name")?;
        let weight = entry.weight.max(0.1);
        let duration = if i == body.children.len() - 1 {
            parent_range.end_ms - cursor
        } else {
            ((weight / total_weight) * parent_duration as f32) as u64
        };

        let end = (cursor + duration).min(parent_range.end_ms);
        let time_range = match TimeRange::new(cursor, end) {
            Ok(r) => r,
            Err(e) => return Err(ApiError::bad_request(e.to_string())),
        };

        let mut node = StoryNode::new_child(&entry.name, child_level, time_range, parent_id);
        node.sort_order = i as u32;
        node.content.notes = entry.outline.clone();
        node.content.status = ContentStatus::NotesOnly;
        if let Some(bt_str) = &entry.beat_type {
            node.beat_type = Some(parse_beat_type(bt_str));
        }

        let node_id = node.id;
        created_nodes.push(
            serde_json::to_value(&node).map_err(|e| ApiError::internal(e.to_string()))?,
        );

        // Add node directly to timeline (bypass add_node validation since we know it's correct).
        project.timeline.nodes.push(node);

        // Tag with parent's arcs.
        for arc_id in &parent_arc_ids {
            project.timeline.tag_node(node_id, *arc_id);
        }

        node_entities.push((
            node_id,
            entry.characters.clone(),
            entry.location.clone(),
            entry.props.clone(),
        ));

        cursor = end;
    }

    // Create or link story bible entities from the plan.
    let mut bible_changed = false;

    fn category_color(cat: &EntityCategory) -> Color {
        match cat {
            EntityCategory::Character => Color::new(100, 149, 237),
            EntityCategory::Location => Color::new(34, 197, 94),
            EntityCategory::Prop => Color::new(249, 115, 22),
            EntityCategory::Theme => Color::new(168, 85, 247),
            EntityCategory::Event => Color::new(239, 68, 68),
        }
    }

    fn ensure_entity(
        bible: &mut eidetic_core::story::bible::StoryBible,
        name: &str,
        category: EntityCategory,
        node_id: NodeId,
    ) -> bool {
        let name_trimmed = name.trim();
        if name_trimmed.is_empty() {
            return false;
        }

        if let Some(entity) = bible
            .entities
            .iter_mut()
            .find(|e| e.name.eq_ignore_ascii_case(name_trimmed))
        {
            if !entity.node_refs.contains(&node_id) {
                entity.node_refs.push(node_id);
            }
            return false;
        }

        let mut entity =
            Entity::new(name_trimmed, category.clone(), category_color(&category));
        entity.details = EntityDetails::default_for(&category);

        if let EntityDetails::Location {
            ref mut int_ext,
            ref mut scene_heading_name,
            ..
        } = entity.details
        {
            let upper = name_trimmed.to_uppercase();
            if upper.starts_with("INT.") || upper.starts_with("INT ") {
                *int_ext = "INT".to_string();
                *scene_heading_name = name_trimmed
                    .get(4..)
                    .map(|s| s.trim_start_matches('.').trim_start_matches(' ').trim())
                    .unwrap_or("")
                    .to_string();
            } else if upper.starts_with("EXT.") || upper.starts_with("EXT ") {
                *int_ext = "EXT".to_string();
                *scene_heading_name = name_trimmed
                    .get(4..)
                    .map(|s| s.trim_start_matches('.').trim_start_matches(' ').trim())
                    .unwrap_or("")
                    .to_string();
            } else {
                *scene_heading_name = name_trimmed.to_string();
            }
        }

        entity.node_refs.push(node_id);
        bible.add_entity(entity);
        true
    }

    for (node_id, characters, location, props) in &node_entities {
        for name in characters {
            ensure_entity(
                &mut project.bible,
                name,
                EntityCategory::Character,
                *node_id,
            );
        }
        if let Some(loc) = location {
            ensure_entity(
                &mut project.bible,
                loc,
                EntityCategory::Location,
                *node_id,
            );
        }
        for name in props {
            ensure_entity(&mut project.bible, name, EntityCategory::Prop, *node_id);
        }
        bible_changed = true;
    }

    // Collect node IDs and outlines for Y.Doc mirroring.
    let children_for_doc: Vec<(NodeId, String)> = node_entities
        .iter()
        .zip(body.children.iter())
        .map(|((nid, _, _, _), entry)| (*nid, entry.outline.clone()))
        .collect();

    drop(guard);

    // Mirror each child node to Y.Doc (ensure + write notes, fire-and-forget).
    for (child_id, outline) in children_for_doc {
        let _ = state
            .doc_tx
            .try_send(DocCommand::EnsureNode { node_id: child_id });
        if !outline.is_empty() {
            let _ = state.doc_tx.try_send(DocCommand::WriteNodeContent {
                node_id: child_id,
                field: ContentField::Notes,
                text: outline,
                author: "ai:decompose".into(),
            });
        }
    }

    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
    if bible_changed {
        let _ = state.events_tx.send(ServerEvent::BibleChanged);
    }
    state.trigger_save();

    Ok(Json(serde_json::json!({
        "ok": true,
        "children": created_nodes,
    })))
}

// ─── Track CRUD ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateTrackRequest {
    level: String,
    label: Option<String>,
}

async fn create_track(
    State(state): State<AppState>,
    Json(body): Json<CreateTrackRequest>,
) -> ApiJson {
    use eidetic_core::timeline::track::Track;

    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let level = match parse_story_level(&body.level) {
        Some(l) => l,
        None => return Err(ApiError::bad_request("invalid level")),
    };

    let mut track = Track::new(level);
    if let Some(label) = body.label {
        validation::validate_name(&label, "track label")?;
        track.label = label;
    }

    let json = serde_json::to_value(&track).map_err(|e| ApiError::internal(e.to_string()))?;
    project.timeline.tracks.push(track);
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    Ok(Json(json))
}

#[derive(Deserialize)]
struct UpdateTrackRequest {
    label: Option<String>,
    collapsed: Option<bool>,
}

async fn update_track(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTrackRequest>,
) -> ApiJson {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    match project.timeline.track_mut(TrackId(id)) {
        Ok(track) => {
            if let Some(label) = body.label {
                validation::validate_name(&label, "track label")?;
                track.label = label;
            }
            if let Some(collapsed) = body.collapsed {
                track.collapsed = collapsed;
            }
            let json = serde_json::to_value(&*track)
                .map_err(|e| ApiError::internal(e.to_string()))?;
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            Ok(Json(json))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

async fn delete_track(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let track_id = TrackId(id);
    let idx = project
        .timeline
        .tracks
        .iter()
        .position(|t| t.id == track_id);
    match idx {
        Some(i) => {
            let track = project.timeline.tracks.remove(i);
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            json_value(&track)
        }
        None => Err(ApiError::not_found("track not found")),
    }
}

// ─── Node-Arc Tagging ──────────────────────────────────────────────

#[derive(Deserialize)]
struct TagNodeArcRequest {
    node_id: Uuid,
    arc_id: Uuid,
}

async fn tag_node_with_arc(
    State(state): State<AppState>,
    Json(body): Json<TagNodeArcRequest>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    project
        .timeline
        .tag_node(NodeId(body.node_id), ArcId(body.arc_id));
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn untag_node_from_arc(
    State(state): State<AppState>,
    Path((node_id, arc_id)): Path<(Uuid, Uuid)>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    project
        .timeline
        .untag_node(NodeId(node_id), ArcId(arc_id));
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── Relationships ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateRelationshipRequest {
    from_node: Uuid,
    to_node: Uuid,
    relationship_type: String,
}

async fn create_relationship(
    State(state): State<AppState>,
    Json(body): Json<CreateRelationshipRequest>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let rel_type = match body.relationship_type.as_str() {
        "convergence" => RelationshipType::Convergence { arc_ids: vec![] },
        "thematic" => RelationshipType::Thematic,
        _ => RelationshipType::Causal,
    };

    let rel = Relationship::new(NodeId(body.from_node), NodeId(body.to_node), rel_type);
    let json = serde_json::to_value(&rel).map_err(|e| ApiError::internal(e.to_string()))?;

    match project.timeline.add_relationship(rel) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            Ok(Json(json))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

async fn delete_relationship(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    match project.timeline.remove_relationship(RelationshipId(id)) {
        Ok(rel) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            json_value(&rel)
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

// ─── Gaps ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GapQuery {
    level: Option<String>,
}

async fn get_gaps(
    State(state): State<AppState>,
    Query(query): Query<GapQuery>,
) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let level = query
                .level
                .as_deref()
                .and_then(parse_story_level)
                .unwrap_or(StoryLevel::Scene);
            let gaps = p
                .timeline
                .find_gaps(level, crate::state::constants::GAP_THRESHOLD_MS);
            json_value(&gaps)
        }
        None => Err(ApiError::no_project()),
    }
}

// ─── Helpers ───────────────────────────────────────────────────────

fn parse_story_level(s: &str) -> Option<StoryLevel> {
    match s {
        "premise" | "Premise" => Some(StoryLevel::Premise),
        "act" | "Act" => Some(StoryLevel::Act),
        "sequence" | "Sequence" => Some(StoryLevel::Sequence),
        "scene" | "Scene" => Some(StoryLevel::Scene),
        "beat" | "Beat" => Some(StoryLevel::Beat),
        _ => None,
    }
}

fn parse_beat_type(s: &str) -> BeatType {
    match s {
        "setup" => BeatType::Setup,
        "complication" => BeatType::Complication,
        "escalation" => BeatType::Escalation,
        "climax" => BeatType::Climax,
        "resolution" => BeatType::Resolution,
        "payoff" => BeatType::Payoff,
        "callback" => BeatType::Callback,
        other => BeatType::Custom(other.to_string()),
    }
}
