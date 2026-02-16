use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::{ArcType, Color, StoryArc};
use eidetic_core::story::bible::{
    Entity, EntityCategory, EntityDetails, EntityRelation, EntitySnapshot, SnapshotOverrides,
};
use eidetic_core::story::progression::analyze_all_arcs;
use eidetic_core::timeline::node::NodeId;

use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/arcs", get(list_arcs))
        .route("/arcs", post(create_arc))
        .route("/arcs/{id}", put(update_arc))
        .route("/arcs/{id}", delete(delete_arc))
        .route("/arcs/progression", get(arc_progression))
        // Bible entity routes
        .route("/bible/entities", get(list_entities))
        .route("/bible/entities", post(create_entity))
        .route("/bible/entities/{id}", get(get_entity))
        .route("/bible/entities/{id}", put(update_entity))
        .route("/bible/entities/{id}", delete(delete_entity))
        .route("/bible/entities/{id}/snapshots", post(add_snapshot))
        .route(
            "/bible/entities/{id}/snapshots/{idx}",
            put(update_snapshot),
        )
        .route(
            "/bible/entities/{id}/snapshots/{idx}",
            delete(delete_snapshot),
        )
        .route(
            "/bible/entities/{id}/node-refs",
            post(add_node_ref),
        )
        .route(
            "/bible/entities/{id}/node-refs/{node_id}",
            delete(remove_node_ref),
        )
        .route(
            "/bible/entities/{id}/relations",
            post(add_relation),
        )
        .route(
            "/bible/entities/{id}/relations/{idx}",
            delete(remove_relation),
        )
        .route("/bible/at", get(resolve_entities_at_time))
}

// ──────────────────────────────────────────────
// Arcs (unchanged)
// ──────────────────────────────────────────────

async fn list_arcs(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => Json(serde_json::to_value(&p.arcs).unwrap()),
        None => Json(serde_json::json!([])),
    }
}

#[derive(Deserialize)]
struct CreateArcRequest {
    name: String,
    arc_type: String,
    color: Option<[u8; 3]>,
}

async fn create_arc(
    State(state): State<AppState>,
    Json(body): Json<CreateArcRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let arc_type = match body.arc_type.as_str() {
        "a_plot" => ArcType::APlot,
        "b_plot" => ArcType::BPlot,
        "c_runner" => ArcType::CRunner,
        other => ArcType::Custom(other.to_string()),
    };

    let color = body
        .color
        .map(|[r, g, b]| Color::new(r, g, b))
        .unwrap_or(Color::new(180, 180, 180));

    let arc = StoryArc::new(body.name, arc_type, color);
    let json = serde_json::to_value(&arc).unwrap();
    project.arcs.push(arc);
    let _ = state.events_tx.send(ServerEvent::StoryChanged);
    state.trigger_save();
    Json(json)
}

async fn update_arc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    if let Some(arc) = project.arcs.iter_mut().find(|a| a.id.0 == id) {
        if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
            arc.name = name.to_string();
        }
        if let Some(desc) = body.get("description").and_then(|v| v.as_str()) {
            arc.description = desc.to_string();
        }
        if let Some(color_arr) = body.get("color").and_then(|v| v.as_array()) {
            if color_arr.len() == 3 {
                if let (Some(r), Some(g), Some(b)) = (
                    color_arr[0].as_u64(),
                    color_arr[1].as_u64(),
                    color_arr[2].as_u64(),
                ) {
                    arc.color = Color::new(r as u8, g as u8, b as u8);
                }
            }
        }
        let json = serde_json::to_value(&*arc).unwrap();
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        Json(json)
    } else {
        Json(serde_json::json!({ "error": "arc not found" }))
    }
}

async fn delete_arc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let before = project.arcs.len();
    project.arcs.retain(|a| a.id.0 != id);
    let deleted = before != project.arcs.len();
    if deleted {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        state.trigger_save();
    }
    Json(serde_json::json!({ "deleted": deleted }))
}

async fn arc_progression(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let progressions = analyze_all_arcs(p);
            Json(serde_json::to_value(&progressions).unwrap())
        }
        None => Json(serde_json::json!([])),
    }
}

// ──────────────────────────────────────────────
// Bible Entities
// ──────────────────────────────────────────────

async fn list_entities(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => Json(serde_json::to_value(&p.bible.entities).unwrap()),
        None => Json(serde_json::json!([])),
    }
}

async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    match project.bible.entity(entity_id) {
        Some(entity) => Json(serde_json::to_value(entity).unwrap()),
        None => Json(serde_json::json!({ "error": "entity not found" })),
    }
}

#[derive(Deserialize)]
struct CreateEntityRequest {
    name: String,
    category: EntityCategory,
    #[serde(default)]
    tagline: Option<String>,
    #[serde(default)]
    description: Option<String>,
    color: Option<[u8; 3]>,
}

async fn create_entity(
    State(state): State<AppState>,
    Json(body): Json<CreateEntityRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let color = body
        .color
        .map(|[r, g, b]| Color::new(r, g, b))
        .unwrap_or(Color::new(200, 200, 200));

    let mut entity = Entity::new(body.name, body.category, color);
    if let Some(tagline) = body.tagline {
        entity.tagline = tagline;
    }
    if let Some(description) = body.description {
        entity.description = description;
    }
    let json = serde_json::to_value(&entity).unwrap();
    project.bible.add_entity(entity);
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(json)
}

async fn update_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
        entity.name = name.to_string();
    }
    if let Some(tagline) = body.get("tagline").and_then(|v| v.as_str()) {
        entity.tagline = tagline.to_string();
    }
    if let Some(desc) = body.get("description").and_then(|v| v.as_str()) {
        entity.description = desc.to_string();
    }
    if let Some(locked) = body.get("locked").and_then(|v| v.as_bool()) {
        entity.locked = locked;
    }
    if let Some(color_arr) = body.get("color").and_then(|v| v.as_array()) {
        if color_arr.len() == 3 {
            if let (Some(r), Some(g), Some(b)) = (
                color_arr[0].as_u64(),
                color_arr[1].as_u64(),
                color_arr[2].as_u64(),
            ) {
                entity.color = Color::new(r as u8, g as u8, b as u8);
            }
        }
    }
    // Update category-specific details if provided.
    if let Some(details) = body.get("details") {
        if let Ok(parsed) = serde_json::from_value::<EntityDetails>(details.clone()) {
            entity.details = parsed;
        }
    }

    let json = serde_json::to_value(&*entity).unwrap();
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(json)
}

async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let deleted = project.bible.remove_entity(entity_id).is_some();
    if deleted {
        let _ = state.events_tx.send(ServerEvent::BibleChanged);
        state.trigger_save();
    }
    Json(serde_json::json!({ "deleted": deleted }))
}

// ──────────────────────────────────────────────
// Snapshots
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct AddSnapshotRequest {
    at_ms: u64,
    source_node_id: Option<Uuid>,
    description: String,
    #[serde(default)]
    state_overrides: Option<SnapshotOverrides>,
}

async fn add_snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AddSnapshotRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    let snapshot = EntitySnapshot {
        at_ms: body.at_ms,
        source_node_id: body.source_node_id.map(NodeId),
        description: body.description,
        state_overrides: body.state_overrides,
    };

    entity.add_snapshot(snapshot);
    let json = serde_json::to_value(&entity.snapshots).unwrap();
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(json)
}

async fn update_snapshot(
    State(state): State<AppState>,
    Path((id, idx)): Path<(Uuid, usize)>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    let Some(snapshot) = entity.snapshots.get_mut(idx) else {
        return Json(serde_json::json!({ "error": "snapshot index out of range" }));
    };

    if let Some(at_ms) = body.get("at_ms").and_then(|v| v.as_u64()) {
        snapshot.at_ms = at_ms;
    }
    if let Some(desc) = body.get("description").and_then(|v| v.as_str()) {
        snapshot.description = desc.to_string();
    }
    if let Some(overrides) = body.get("state_overrides") {
        snapshot.state_overrides = serde_json::from_value(overrides.clone()).ok();
    }

    // Re-sort if at_ms changed.
    entity.sort_snapshots();

    let json = serde_json::to_value(&entity.snapshots).unwrap();
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(json)
}

async fn delete_snapshot(
    State(state): State<AppState>,
    Path((id, idx)): Path<(Uuid, usize)>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    if idx >= entity.snapshots.len() {
        return Json(serde_json::json!({ "error": "snapshot index out of range" }));
    }

    entity.snapshots.remove(idx);
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(serde_json::json!({ "deleted": true }))
}

// ──────────────────────────────────────────────
// Node References
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct AddNodeRefRequest {
    node_id: Uuid,
}

async fn add_node_ref(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AddNodeRefRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    let node_id = NodeId(body.node_id);
    if !entity.node_refs.contains(&node_id) {
        entity.node_refs.push(node_id);
    }

    let json = serde_json::to_value(&entity.node_refs).unwrap();
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(json)
}

async fn remove_node_ref(
    State(state): State<AppState>,
    Path((id, node_id)): Path<(Uuid, Uuid)>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    let target = NodeId(node_id);
    entity.node_refs.retain(|n| *n != target);

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(serde_json::json!({ "deleted": true }))
}

// ──────────────────────────────────────────────
// Relations
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct AddRelationRequest {
    target_entity_id: Uuid,
    label: String,
}

async fn add_relation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AddRelationRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    entity.relations.push(EntityRelation {
        target_entity_id: eidetic_core::story::bible::EntityId(body.target_entity_id),
        label: body.label,
    });

    let json = serde_json::to_value(&entity.relations).unwrap();
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(json)
}

async fn remove_relation(
    State(state): State<AppState>,
    Path((id, idx)): Path<(Uuid, usize)>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let entity_id = eidetic_core::story::bible::EntityId(id);
    let Some(entity) = project.bible.entity_mut(entity_id) else {
        return Json(serde_json::json!({ "error": "entity not found" }));
    };

    if idx >= entity.relations.len() {
        return Json(serde_json::json!({ "error": "relation index out of range" }));
    }

    entity.relations.remove(idx);
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    state.trigger_save();
    Json(serde_json::json!({ "deleted": true }))
}

// ──────────────────────────────────────────────
// Resolve entities at a point in time
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct ResolveAtQuery {
    time_ms: u64,
}

async fn resolve_entities_at_time(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ResolveAtQuery>,
) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Json(serde_json::json!([]));
    };

    let resolved: Vec<serde_json::Value> = project
        .bible
        .entities
        .iter()
        .map(|entity| {
            let snapshot = entity.active_snapshot_at(query.time_ms);
            serde_json::json!({
                "entity": entity,
                "active_snapshot": snapshot,
                "compact_text": entity.to_prompt_text(query.time_ms),
            })
        })
        .collect();

    Json(serde_json::json!(resolved))
}
