use super::router;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{FieldValue, ObjectKind};
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

#[tokio::test]
async fn create_timeline_relationship_command_returns_timeline_render_projection() {
    let path = temp_db_path("creates-timeline-relationship");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_relationship_command_body(relationship_id, from_node, to_node);

    let response = app
        .oneshot(create_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let relationships = value["projection"]["payload"]["relationships"]
        .as_array()
        .expect("timeline relationships");
    assert!(relationships.iter().any(|relationship| {
        relationship["relationship_id"] == relationship_id.0.to_string()
            && relationship["from_node_id"] == from_node.0.to_string()
            && relationship["to_node_id"] == to_node.0.to_string()
    }));

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineRelationship,
        &relationship_id.0.to_string(),
    )
    .expect("timeline relationship revisions");
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Create
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "from_node_id"
                && field.new_value == Some(FieldValue::Text(from_node.0.to_string())))
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "relationship_type"
                && field.new_value == Some(FieldValue::Text("\"Thematic\"".to_string())))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_relationship_command_replays_duplicate_command() {
    let path = temp_db_path("creates-timeline-relationship-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_relationship_command_body_with_id(
        uuid::Uuid::new_v4(),
        relationship_id,
        from_node,
        to_node,
    );

    let first = app
        .clone()
        .oneshot(create_timeline_relationship_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(create_timeline_relationship_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    let relationships = value["projection"]["payload"]["relationships"]
        .as_array()
        .expect("timeline relationships");
    assert_eq!(
        relationships
            .iter()
            .filter(|relationship| relationship["relationship_id"] == relationship_id.0.to_string())
            .count(),
        1
    );

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineRelationship,
        &relationship_id.0.to_string(),
    )
    .expect("timeline relationship revisions");
    assert_eq!(revisions.len(), 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_relationship_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("creates-timeline-relationship-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let command_id = uuid::Uuid::new_v4();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let original = create_timeline_relationship_command_body_with_id(
        command_id,
        eidetic_core::timeline::relationship::RelationshipId::new(),
        from_node,
        to_node,
    );
    let conflicting = create_timeline_relationship_command_body_with_id(
        command_id,
        eidetic_core::timeline::relationship::RelationshipId::new(),
        to_node,
        from_node,
    );

    let first = app
        .clone()
        .oneshot(create_timeline_relationship_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(create_timeline_relationship_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_relationship_command_rejects_unknown_endpoint() {
    let path = temp_db_path("rejects-unknown-timeline-relationship-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let to_node = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_relationship_command_body(
        eidetic_core::timeline::relationship::RelationshipId::new(),
        eidetic_core::timeline::node::NodeId::new(),
        to_node,
    );

    let response = app
        .oneshot(create_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_relationship_command_returns_timeline_render_projection() {
    let path = temp_db_path("deletes-timeline-relationship");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let mut relationship = eidetic_core::timeline::relationship::Relationship::new(
        from_node,
        to_node,
        eidetic_core::timeline::relationship::RelationshipType::Thematic,
    );
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    relationship.id = relationship_id;
    project.timeline.add_relationship(relationship).unwrap();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_relationship_command_body(relationship_id);

    let response = app
        .oneshot(delete_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let relationships = value["projection"]["payload"]["relationships"]
        .as_array()
        .expect("timeline relationships");
    assert!(
        relationships
            .iter()
            .all(|relationship| relationship["relationship_id"] != relationship_id.0.to_string())
    );

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineRelationship,
        &relationship_id.0.to_string(),
    )
    .expect("timeline relationship revisions");
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Delete
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "from_node_id"
                && field.old_value == Some(FieldValue::Text(from_node.0.to_string())))
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "relationship_type"
                && field.old_value == Some(FieldValue::Text("\"Thematic\"".to_string())))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_relationship_command_replays_duplicate_command() {
    let path = temp_db_path("deletes-timeline-relationship-duplicate");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let mut relationship = eidetic_core::timeline::relationship::Relationship::new(
        from_node,
        to_node,
        eidetic_core::timeline::relationship::RelationshipType::Thematic,
    );
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    relationship.id = relationship_id;
    project.timeline.add_relationship(relationship).unwrap();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body =
        delete_timeline_relationship_command_body_with_id(uuid::Uuid::new_v4(), relationship_id);

    let first = app
        .clone()
        .oneshot(delete_timeline_relationship_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(delete_timeline_relationship_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    let relationships = value["projection"]["payload"]["relationships"]
        .as_array()
        .expect("timeline relationships");
    assert!(
        relationships
            .iter()
            .all(|relationship| relationship["relationship_id"] != relationship_id.0.to_string())
    );

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineRelationship,
        &relationship_id.0.to_string(),
    )
    .expect("timeline relationship revisions");
    assert_eq!(revisions.len(), 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_relationship_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("deletes-timeline-relationship-conflict");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let mut relationship = eidetic_core::timeline::relationship::Relationship::new(
        from_node,
        to_node,
        eidetic_core::timeline::relationship::RelationshipType::Thematic,
    );
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    relationship.id = relationship_id;
    project.timeline.add_relationship(relationship).unwrap();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let command_id = uuid::Uuid::new_v4();
    let original = delete_timeline_relationship_command_body_with_id(command_id, relationship_id);
    let conflicting = delete_timeline_relationship_command_body_with_id(
        command_id,
        eidetic_core::timeline::relationship::RelationshipId::new(),
    );

    let first = app
        .clone()
        .oneshot(delete_timeline_relationship_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(delete_timeline_relationship_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_relationship_command_rejects_unknown_relationship() {
    let path = temp_db_path("rejects-unknown-timeline-relationship-delete");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_relationship_command_body(
        eidetic_core::timeline::relationship::RelationshipId::new(),
    );

    let response = app
        .oneshot(delete_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

fn create_timeline_relationship_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/create-relationship")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn delete_timeline_relationship_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/delete-relationship")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_timeline_relationship_command_body(
    relationship_id: eidetic_core::timeline::relationship::RelationshipId,
    from_node_id: eidetic_core::timeline::node::NodeId,
    to_node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    create_timeline_relationship_command_body_with_id(
        uuid::Uuid::new_v4(),
        relationship_id,
        from_node_id,
        to_node_id,
    )
}

fn create_timeline_relationship_command_body_with_id(
    command_id: uuid::Uuid,
    relationship_id: eidetic_core::timeline::relationship::RelationshipId,
    from_node_id: eidetic_core::timeline::node::NodeId,
    to_node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": command_id,
        "payload": {
            "relationship_id": relationship_id,
            "from_node_id": from_node_id,
            "to_node_id": to_node_id,
            "relationship_type": eidetic_core::timeline::relationship::RelationshipType::Thematic,
        }
    })
}

fn delete_timeline_relationship_command_body(
    relationship_id: eidetic_core::timeline::relationship::RelationshipId,
) -> serde_json::Value {
    delete_timeline_relationship_command_body_with_id(uuid::Uuid::new_v4(), relationship_id)
}

fn delete_timeline_relationship_command_body_with_id(
    command_id: uuid::Uuid,
    relationship_id: eidetic_core::timeline::relationship::RelationshipId,
) -> serde_json::Value {
    json!({
        "id": command_id,
        "payload": {
            "relationship_id": relationship_id,
        }
    })
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-command-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}
