use std::path::PathBuf;

#[cfg(test)]
use eidetic_core::ai::backend::GenerateChildrenRequest;
use eidetic_core::ai::backend::GenerateRequest;
use eidetic_core::contracts::{AiBibleContextProjection, ProjectionEnvelope};
use eidetic_core::timeline::node::NodeId;

use crate::state::AppState;

pub(crate) async fn active_sqlite_project(
    state: &AppState,
) -> Result<(eidetic_core::Project, PathBuf), String> {
    let Some(project_path) = state.project_database.active_path() else {
        return Err("no project loaded".to_string());
    };
    if state.project.lock().is_none() {
        return Err("no project loaded".to_string());
    }
    let (project, _) = crate::persistence::load_project(&project_path).await?;
    Ok((project, project_path))
}

async fn load_ai_bible_context_projection(
    path: PathBuf,
    node_id: NodeId,
) -> Result<ProjectionEnvelope<AiBibleContextProjection>, String> {
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| format!("open AI bible context database failed: {e}"))?;
        crate::ai_context_projection::load_ai_bible_context_projection(&conn, node_id)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("AI bible context projection task failed: {e}"))?
}

pub(crate) async fn attach_ai_bible_context(
    request: &mut GenerateRequest,
    path: PathBuf,
    node_id: NodeId,
) -> Result<(), String> {
    request.bible_context = Some(load_ai_bible_context_projection(path, node_id).await?);
    Ok(())
}

#[cfg(test)]
pub(crate) async fn attach_ai_bible_context_to_children(
    request: &mut GenerateChildrenRequest,
    path: PathBuf,
    node_id: NodeId,
) -> Result<(), String> {
    request.bible_context = Some(load_ai_bible_context_projection(path, node_id).await?);
    Ok(())
}
