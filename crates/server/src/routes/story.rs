use axum::Router;
use axum::extract::State;
use axum::routing::get;

use eidetic_core::story::progression::analyze_all_arcs;

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/arcs/progression", get(arc_progression))
}

// ──────────────────────────────────────────────
// Arc progression
// ──────────────────────────────────────────────

async fn arc_progression(State(state): State<AppState>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let progressions = analyze_all_arcs(p);
            json_value(&progressions)
        }
        None => Err(ApiError::no_project()),
    }
}
