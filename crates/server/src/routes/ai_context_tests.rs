use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::ai::prompt::{build_generate_children_request, build_generate_request};
use eidetic_core::contracts::{
    BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartId, BibleGraphPartKey,
    BibleGraphSchemaKey, CommandEnvelope, CreateBibleGraphNodeCommand, FieldValue,
    SetBibleGraphFieldCommand,
};
use eidetic_core::timeline::node::ContentStatus;
use tower::util::ServiceExt;
use uuid::Uuid;

use super::{attach_ai_bible_context, attach_ai_bible_context_to_children, router};
use crate::prompt_format::build_decompose_prompt;
use crate::state::AppState;

#[tokio::test]
async fn preview_context_includes_graph_backed_bible_context() {
    let path = std::env::temp_dir().join(format!("eidetic-ai-context-bible-{}.db", Uuid::new_v4()));
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("AI Bible Context Test");
    let node_id = project.timeline.nodes[0].id;
    let node = project.timeline.node_mut(node_id).expect("target node");
    node.content.notes = "Use backend-owned bible context".to_string();
    node.content.status = ContentStatus::NotesOnly;
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed project database");
    seed_bible_context(&path);
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/ai/context/{}", node_id.0))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    let user_prompt = value["user"].as_str().expect("user prompt");
    assert!(user_prompt.contains("STORY BIBLE CONTEXT"));
    assert!(user_prompt.contains("Ada"));
    assert!(user_prompt.contains("Reluctant detective"));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn attach_ai_bible_context_loads_projection_for_generation_requests() {
    let path = std::env::temp_dir().join(format!(
        "eidetic-generate-bible-context-{}.db",
        Uuid::new_v4()
    ));
    let mut project = Template::MultiCam.build_project("Generate Bible Context Test");
    let node_id = project.timeline.nodes[0].id;
    let node = project.timeline.node_mut(node_id).expect("target node");
    node.content.notes = "Use backend-owned bible context".to_string();
    node.content.status = ContentStatus::NotesOnly;
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed project database");
    seed_bible_context(&path);
    let mut request = build_generate_request(&project, node_id).expect("generate request");

    attach_ai_bible_context(&mut request, path.clone(), node_id)
        .await
        .expect("attach bible context");

    let bible_context = request.bible_context.expect("bible context projection");
    assert_eq!(bible_context.payload.nodes[0].name, "Ada");
    assert_eq!(
        bible_context.payload.nodes[0].fields[0].value,
        FieldValue::Text("Reluctant detective".to_string())
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn child_decomposition_prompt_includes_graph_backed_bible_context() {
    let path =
        std::env::temp_dir().join(format!("eidetic-child-bible-context-{}.db", Uuid::new_v4()));
    let mut project = Template::MultiCam.build_project("Child Bible Context Test");
    let node_id = project.timeline.nodes[0].id;
    let node = project.timeline.node_mut(node_id).expect("target node");
    node.content.notes = "Break this story using backend-owned bible context".to_string();
    node.content.status = ContentStatus::NotesOnly;
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed project database");
    seed_bible_context(&path);
    let mut request =
        build_generate_children_request(&project, node_id).expect("generate children request");

    attach_ai_bible_context_to_children(&mut request, path.clone(), node_id)
        .await
        .expect("attach bible context");
    let prompt = build_decompose_prompt(&request);

    assert!(prompt.user.contains("STORY BIBLE CONTEXT"));
    assert!(prompt.user.contains("Ada"));
    assert!(prompt.user.contains("Reluctant detective"));

    let _ = std::fs::remove_file(path);
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn seed_bible_context(path: &std::path::Path) {
    let mut conn = crate::sqlite::open_write_connection(path).expect("open sqlite");
    crate::bible_graph_command::apply_create_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: "Ada".to_string(),
            sort_order: 10,
        }),
        100,
    )
    .unwrap();
    crate::bible_graph_command::apply_set_bible_graph_field(
        &mut conn,
        &CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            part_id: BibleGraphPartId::new("part.character.profile").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".to_string(),
            part_sort_order: 10,
            field_id: BibleGraphFieldId::new("field.character.tagline").unwrap(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text("Reluctant detective".to_string())),
            field_sort_order: 20,
        }),
        200,
    )
    .unwrap();
}
