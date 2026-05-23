mod ai_commands;
pub mod bevy_graph_host;
mod commands;
mod desktop_events;
mod error;
mod export_commands;
mod graph_renderer_commands;
mod health;
mod model_commands;
mod project_commands;
mod projections;
mod reference_commands;
mod renderer_window;

use bevy_graph_host::DesktopBibleGraphRendererOwner;
use eidetic_server::state::AppState;
use graph_renderer_commands::GraphRendererProjectionRequestState;
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
pub struct DesktopSmokeReport {
    status: &'static str,
    boundary: &'static str,
    backend_runtime: &'static str,
    active_backend_tasks: usize,
    model_library_configured: bool,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            app.manage(
                DesktopBibleGraphRendererOwner::start()
                    .expect("failed to start Bevy bible graph renderer owner"),
            );
            app.manage(GraphRendererProjectionRequestState::default());
            desktop_events::spawn_server_event_bridge(app.handle().clone(), &app_state);
            desktop_events::spawn_graph_renderer_projection_bridge(
                app.handle().clone(),
                &app_state,
            );
            app.manage(app_state);
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                if let Some(graph_owner) = window.try_state::<DesktopBibleGraphRendererOwner>() {
                    let _ = graph_owner.stop();
                }
                let state = window.state::<AppState>();
                state.shutdown_tasks();
            }
        })
        .invoke_handler(tauri::generate_handler![
            health::desktop_health,
            project_commands::project_create,
            project_commands::project_get,
            project_commands::project_update,
            project_commands::project_save,
            project_commands::project_load,
            project_commands::project_list,
            ai_commands::ai_status,
            ai_commands::ai_config_update,
            ai_commands::ai_context_preview,
            ai_commands::ai_generate_content,
            ai_commands::ai_generate_children,
            ai_commands::ai_generate_batch,
            model_commands::model_list,
            export_commands::export_pdf,
            graph_renderer_commands::graph_renderer_open,
            graph_renderer_commands::graph_renderer_focus,
            graph_renderer_commands::graph_renderer_close,
            graph_renderer_commands::graph_renderer_status,
            graph_renderer_commands::graph_renderer_set_projection,
            graph_renderer_commands::graph_renderer_drain_commands,
            graph_renderer_commands::graph_renderer_visual_snapshot,
            reference_commands::reference_list,
            reference_commands::reference_upload,
            reference_commands::reference_delete,
            commands::object_script_story::command_object_field,
            commands::object_script_story::command_script_block,
            commands::object_script_story::command_script_lock,
            commands::object_script_story::command_story_create,
            commands::object_script_story::command_story_update,
            commands::object_script_story::command_story_delete,
            commands::bible::command_bible_graph_node,
            commands::bible::command_bible_graph_field,
            commands::bible::command_bible_graph_edge,
            commands::bible::command_bible_graph_snapshot_field,
            commands::bible::command_bible_graph_roots,
            commands::context::command_context_evaluation,
            commands::semantic::command_bible_reference_proposal_create,
            commands::semantic::command_bible_reference_proposal_reject,
            commands::semantic::command_bible_reference_proposal_accept,
            commands::semantic::command_propagation_proposal_create,
            commands::semantic::command_propagation_proposal_reject,
            commands::semantic::command_propagation_proposal_update,
            commands::semantic::command_propagation_proposal_accept,
            commands::timeline::command_timeline_create_node,
            commands::timeline::command_timeline_node_range,
            commands::timeline::command_timeline_node_lock,
            commands::timeline::command_timeline_node_notes,
            commands::timeline::command_timeline_delete_node,
            commands::timeline::command_timeline_create_relationship,
            commands::timeline::command_timeline_delete_relationship,
            commands::timeline::command_timeline_apply_children,
            commands::timeline::command_timeline_split_node,
            projections::story_script::projection_object_field,
            projections::story_script::projection_script_document,
            projections::bible::projection_bible_graph_node,
            projections::bible::projection_bible_graph_nodes,
            projections::bible::projection_bible_graph_schemas,
            projections::bible::projection_bible_render_graph,
            projections::context::projection_context_influence,
            projections::context::projection_context_stack,
            projections::semantic::projection_bible_reference_proposals,
            projections::semantic::projection_propagation_proposals,
            projections::semantic::projection_semantic_dependencies,
            projections::semantic::projection_child_plans,
            projections::story_script::projection_story_arcs,
            projections::story_script::projection_story_arc_progression,
            projections::story_script::projection_change_review,
            projections::timeline::projection_timeline_render,
            projections::timeline::projection_selected_node
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Eidetic desktop application");
}

pub fn smoke_report() -> DesktopSmokeReport {
    let app_state = tauri::async_runtime::block_on(AppState::new());
    let report = DesktopSmokeReport {
        status: "ok",
        boundary: "tauri",
        backend_runtime: "initialized",
        active_backend_tasks: app_state.task_supervisor.active_task_count(),
        model_library_configured: app_state.model_library.is_some(),
    };
    app_state.shutdown_tasks();
    report
}

pub fn smoke_report_json() -> Result<String, serde_json::Error> {
    serde_json::to_string(&smoke_report())
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_report_initializes_backend_runtime() {
        let report = super::smoke_report();

        assert_eq!(report.status, "ok");
        assert_eq!(report.boundary, "tauri");
        assert_eq!(report.backend_runtime, "initialized");
        assert!(report.active_backend_tasks >= 2);
    }
}
