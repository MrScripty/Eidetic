use eidetic_core::contracts::ScriptDocumentId;

use crate::backend_error::BackendError;
use crate::export::generate_screenplay_pdf;
use crate::history_store::HistoryStoreError;
use crate::script_store;
use crate::state::AppState;

const MAIN_SCRIPT_DOCUMENT_ID: &str = "script.document.main";

pub async fn export_pdf(state: &AppState) -> Result<Vec<u8>, BackendError> {
    let project_name = {
        let guard = state.project.lock();
        match guard.as_ref() {
            Some(project) => project.name.clone(),
            None => return Err(BackendError::BadRequest("no project loaded".to_string())),
        }
    };
    let path = state
        .project_database
        .active_path()
        .ok_or_else(|| BackendError::BadRequest("no project loaded".to_string()))?;

    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::Internal(error.to_string()))?;
        script_store::create_schema(&conn).map_err(map_history_error)?;
        let document_id = ScriptDocumentId::new(MAIN_SCRIPT_DOCUMENT_ID)
            .map_err(|error| BackendError::BadRequest(error.to_string()))?;
        let projection = script_store::load_document_projection(&conn, &document_id)
            .map_err(map_history_error)?
            .ok_or_else(|| BackendError::NotFound("script document not found".to_string()))?;
        generate_screenplay_pdf(&project_name, &projection).map_err(BackendError::Internal)
    })
    .await
    .map_err(|error| BackendError::Internal(format!("PDF export task failed: {error}")))?
}

fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::Conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::BadRequest(message),
        HistoryStoreError::MissingColumn(message) => BackendError::Internal(message.to_string()),
        HistoryStoreError::Sqlite(error) => BackendError::Internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::BadRequest(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::export_pdf;
    use crate::state::AppState;
    use eidetic_core::Template;
    use uuid::Uuid;

    #[tokio::test]
    async fn export_pdf_requires_loaded_project() {
        let state = AppState::new().await;

        let error = export_pdf(&state).await.expect_err("missing project");

        assert_eq!(error.message(), "no project loaded");
    }

    #[tokio::test]
    async fn export_pdf_requires_script_document_projection() {
        let path = std::env::temp_dir().join(format!(
            "eidetic-export-service-missing-script-{}.db",
            Uuid::new_v4()
        ));
        let state = AppState::new().await;
        *state.project.lock() = Some(Template::MultiCam.build_project("Export Test"));
        *state.project_path.lock() = Some(path.clone());

        let error = export_pdf(&state)
            .await
            .expect_err("missing script document");

        assert_eq!(error.message(), "script document not found");

        let _ = std::fs::remove_file(path);
    }
}
