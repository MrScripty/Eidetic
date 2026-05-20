use eidetic_server::backend_error::BackendError;
use eidetic_server::project_service;
use eidetic_server::state::AppState;
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct DesktopHealth {
    status: &'static str,
    boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct CommandError {
    kind: &'static str,
    message: String,
}

impl From<BackendError> for CommandError {
    fn from(error: BackendError) -> Self {
        let kind = match error {
            BackendError::NotFound(_) => "not_found",
            BackendError::BadRequest(_) => "bad_request",
            BackendError::Conflict(_) => "conflict",
            BackendError::Internal(_) => "internal",
        };

        Self {
            kind,
            message: error.message().to_string(),
        }
    }
}

#[tauri::command]
fn desktop_health() -> DesktopHealth {
    DesktopHealth {
        status: "ok",
        boundary: "tauri",
    }
}

#[tauri::command]
fn project_get(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, CommandError> {
    project_service::get_project(&state).map_err(CommandError::from)
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![desktop_health, project_get])
        .run(tauri::generate_context!())
        .expect("failed to run Eidetic desktop application");
}
