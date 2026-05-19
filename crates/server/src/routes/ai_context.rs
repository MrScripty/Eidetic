use axum::Json;
use axum::extract::{Path, State};
use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::timeline::node::NodeId;
use uuid::Uuid;

use crate::prompt_format::build_chat_prompt;
use crate::state::AppState;

use super::{active_sqlite_project, attach_ai_bible_context};

/// Preview the formatted AI context/prompt for a node without generating.
pub(super) async fn preview_context(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(id);

    let (project, project_path) = match active_sqlite_project(&state).await {
        Ok(project) => project,
        Err(error) => return Json(serde_json::json!({ "error": error })),
    };

    let mut request = match build_generate_request(&project, node_id) {
        Ok(req) => req,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    if let Err(error) = attach_ai_bible_context(&mut request, project_path, node_id).await {
        return Json(serde_json::json!({ "error": error }));
    }

    let prompt = build_chat_prompt(&request);

    Json(serde_json::json!({
        "system": prompt.system,
        "user": prompt.user,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::ai::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use eidetic_core::Template;
    use eidetic_core::timeline::node::ContentStatus;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn preview_context_hydrates_story_arcs_from_sqlite_when_project_mirror_is_stale() {
        let path =
            std::env::temp_dir().join(format!("eidetic-ai-context-arcs-{}.db", Uuid::new_v4()));
        let state = AppState::new().await;
        let mut project = Template::MultiCam.build_project("AI Context Test");
        let node_arc = project.timeline.node_arcs[0].clone();
        let node = project
            .timeline
            .node_mut(node_arc.node_id)
            .expect("tagged node");
        node.content.notes = "SQLite-only rain argument".to_string();
        node.content.status = ContentStatus::NotesOnly;
        let arc_name = project
            .arcs
            .iter()
            .find(|arc| arc.id == node_arc.arc_id)
            .expect("tagged arc")
            .name
            .clone();
        crate::persistence::save_project(&project, &path, None)
            .await
            .expect("seed project database");
        project.arcs.clear();
        let node = project
            .timeline
            .node_mut(node_arc.node_id)
            .expect("tagged node");
        node.content.notes.clear();
        node.content.status = ContentStatus::Empty;
        *state.project.lock() = Some(project);
        *state.project_path.lock() = Some(path.clone());
        let app = router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/ai/context/{}", node_arc.node_id.0))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let value = response_json(response).await;
        assert!(
            value["user"]
                .as_str()
                .expect("user prompt")
                .contains(&arc_name),
            "prompt should include arc name loaded from sqlite"
        );
        assert!(
            value["user"]
                .as_str()
                .expect("user prompt")
                .contains("SQLite-only rain argument"),
            "prompt should include node notes loaded from sqlite"
        );

        let _ = std::fs::remove_file(path);
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body bytes");
        serde_json::from_slice(&body).expect("json response")
    }
}
