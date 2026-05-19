use super::router;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{BibleReferenceKind, FieldValue, ObjectKind, SemanticProposalStatus};
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

#[tokio::test]
async fn apply_timeline_children_command_returns_timeline_render_projection() {
    let path = temp_db_path("applies-timeline-children");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let original_child = project
        .timeline
        .children_of(parent.id)
        .first()
        .expect("original child")
        .id;
    let first_child_id = eidetic_core::timeline::node::NodeId::new();
    let second_child_id = eidetic_core::timeline::node::NodeId::new();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed child current state");
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = apply_timeline_children_command_body(parent.id, first_child_id, second_child_id);

    let response = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != original_child.0.to_string())
    );
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == first_child_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "First child"
    }));
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == second_child_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "Second child"
    }));

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let original_child_count = conn
        .query_row(
            "SELECT COUNT(*) FROM nodes WHERE id = ?1",
            [original_child.0.to_string()],
            |row| row.get::<_, i64>(0),
        )
        .expect("original child count");
    assert_eq!(original_child_count, 0);
    let first_child = conn
        .query_row(
            "SELECT parent_id, name, content_json FROM nodes WHERE id = ?1",
            [first_child_id.0.to_string()],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .expect("persisted first child");
    assert_eq!(first_child.0, Some(parent.id.0.to_string()));
    assert_eq!(first_child.1, "First child");
    let first_child_content: serde_json::Value =
        serde_json::from_str(&first_child.2).expect("first child content");
    assert_eq!(first_child_content["notes"], "First outline");
    assert_eq!(first_child_content["status"], "NotesOnly");

    let deleted_revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &original_child.0.to_string(),
    )
    .expect("deleted child revisions");
    assert_eq!(deleted_revisions.len(), 1);
    assert_eq!(
        deleted_revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Delete
    );
    let created_revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &first_child_id.0.to_string(),
    )
    .expect("created child revisions");
    assert_eq!(created_revisions.len(), 1);
    assert_eq!(
        created_revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Create
    );
    assert!(
        created_revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "notes"
                && field.old_value.is_none()
                && field.new_value == Some(FieldValue::Text("First outline".to_string())))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_records_bible_reference_proposals() {
    let path = temp_db_path("applies-children-with-bible-references");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let first_child_id = eidetic_core::timeline::node::NodeId::new();
    let second_child_id = eidetic_core::timeline::node::NodeId::new();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed child current state");
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let mut body = apply_timeline_children_command_body(parent.id, first_child_id, second_child_id);
    body["payload"]["children"][0]["characters"] = json!(["Ada", "Bob", "Ada", " "]);
    body["payload"]["children"][0]["location"] = json!("Storm Harbor");
    body["payload"]["children"][0]["props"] = json!(["Signal ring"]);

    let response = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    let proposals = crate::semantic_proposal_store::load_bible_reference_proposals(&conn).unwrap();

    assert_eq!(proposals.len(), 4);
    assert!(
        proposals
            .iter()
            .all(|proposal| proposal.status == SemanticProposalStatus::Pending)
    );
    assert!(
        proposals
            .iter()
            .all(|proposal| proposal.source_node_id == first_child_id)
    );
    assert!(proposals.iter().any(|proposal| {
        proposal.reference_kind == BibleReferenceKind::Character && proposal.reference_text == "Ada"
    }));
    assert!(proposals.iter().any(|proposal| {
        proposal.reference_kind == BibleReferenceKind::Location
            && proposal.reference_text == "Storm Harbor"
    }));
    assert!(proposals.iter().any(|proposal| {
        proposal.reference_kind == BibleReferenceKind::Prop
            && proposal.reference_text == "Signal ring"
    }));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_replays_duplicate_command() {
    let path = temp_db_path("applies-timeline-children-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let stale_project = project.clone();
    let parent = project.timeline.nodes[0].clone();
    let original_child = project
        .timeline
        .children_of(parent.id)
        .first()
        .expect("original child")
        .id;
    let first_child_id = eidetic_core::timeline::node::NodeId::new();
    let second_child_id = eidetic_core::timeline::node::NodeId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
    let body = apply_timeline_children_command_body_with_id(
        uuid::Uuid::new_v4(),
        parent.id,
        first_child_id,
        second_child_id,
    );

    let first = app
        .clone()
        .oneshot(apply_timeline_children_command_request(body.clone()))
        .await
        .expect("first route response");
    *state.project.lock() = Some(stale_project);
    let second = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != original_child.0.to_string())
    );
    assert!(
        clips
            .iter()
            .any(|clip| clip["node_id"] == first_child_id.0.to_string())
    );

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &first_child_id.0.to_string(),
    )
    .expect("created child revisions");
    assert_eq!(revisions.len(), 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_validates_against_sqlite_when_project_mirror_is_stale() {
    let path = temp_db_path("applies-timeline-children-stale-mirror");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let original_child = project
        .timeline
        .children_of(parent.id)
        .first()
        .expect("original child")
        .id;
    let first_child_id = eidetic_core::timeline::node::NodeId::new();
    let second_child_id = eidetic_core::timeline::node::NodeId::new();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed child current state");
    let mut stale_project = project;
    stale_project
        .timeline
        .remove_node(parent.id)
        .expect("make mirror stale");
    *state.project.lock() = Some(stale_project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = apply_timeline_children_command_body(parent.id, first_child_id, second_child_id);

    let response = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != original_child.0.to_string())
    );
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == first_child_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "First child"
    }));
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == second_child_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "Second child"
    }));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("applies-timeline-children-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let command_id = uuid::Uuid::new_v4();
    let original = apply_timeline_children_command_body_with_id(
        command_id,
        parent.id,
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    );
    let conflicting = apply_timeline_children_command_body_with_id(
        command_id,
        parent.id,
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    );

    let first = app
        .clone()
        .oneshot(apply_timeline_children_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(apply_timeline_children_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_rejects_unknown_parent() {
    let path = temp_db_path("rejects-unknown-timeline-children-parent");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = apply_timeline_children_command_body(
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    );

    let response = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

fn apply_timeline_children_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/apply-children")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn apply_timeline_children_command_body(
    parent_id: eidetic_core::timeline::node::NodeId,
    first_child_id: eidetic_core::timeline::node::NodeId,
    second_child_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    apply_timeline_children_command_body_with_id(
        uuid::Uuid::new_v4(),
        parent_id,
        first_child_id,
        second_child_id,
    )
}

fn apply_timeline_children_command_body_with_id(
    command_id: uuid::Uuid,
    parent_id: eidetic_core::timeline::node::NodeId,
    first_child_id: eidetic_core::timeline::node::NodeId,
    second_child_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": command_id,
        "payload": {
            "parent_id": parent_id,
            "children": [
                {
                    "node_id": first_child_id,
                    "name": "First child",
                    "outline": "First outline",
                    "weight": 1.0,
                    "beat_type": null,
                },
                {
                    "node_id": second_child_id,
                    "name": "Second child",
                    "outline": "Second outline",
                    "weight": 1.0,
                    "beat_type": null,
                }
            ]
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
