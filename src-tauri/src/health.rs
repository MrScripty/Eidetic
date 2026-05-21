use serde::Serialize;

#[derive(Serialize)]
pub struct DesktopHealth {
    status: &'static str,
    boundary: &'static str,
}

#[tauri::command]
pub fn desktop_health() -> DesktopHealth {
    DesktopHealth {
        status: "ok",
        boundary: "tauri",
    }
}
