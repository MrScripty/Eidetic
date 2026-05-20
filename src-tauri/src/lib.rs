use eidetic_server::backend_error::BackendError;
use eidetic_server::project_service::{
    self, CreateProjectRequest, LoadProjectRequest, SaveProjectRequest, UpdateProjectRequest,
};
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
async fn project_create(
    app: tauri::AppHandle,
    name: String,
    template: String,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    project_service::create_project(&state, CreateProjectRequest { name, template })
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
fn project_get(app: tauri::AppHandle) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>();
    project_service::get_project(&state).map_err(CommandError::from)
}

#[tauri::command]
fn project_update(
    app: tauri::AppHandle,
    name: Option<String>,
    premise: Option<String>,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>();
    project_service::update_project(&state, UpdateProjectRequest { name, premise })
        .map_err(CommandError::from)
}

#[tauri::command]
async fn project_save(
    app: tauri::AppHandle,
    path: Option<String>,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    project_service::save_project(&state, SaveProjectRequest { path })
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn project_load(
    app: tauri::AppHandle,
    path: String,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    project_service::load_project(&state, LoadProjectRequest { path })
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn project_list() -> serde_json::Value {
    project_service::list_projects().await
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_health,
            project_create,
            project_get,
            project_update,
            project_save,
            project_load,
            project_list
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Eidetic desktop application");
}
