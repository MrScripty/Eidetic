mod ai_commands;
pub mod bevy_graph_host;
pub mod bevy_timeline_host;
mod bevy_timeline_owner;
mod commands;
mod desktop_events;
mod desktop_smoke;
mod error;
mod export_commands;
mod graph_renderer_commands;
mod graph_renderer_projection;
mod health;
mod model_commands;
mod project_commands;
mod projections;
mod reference_commands;
mod renderer_window;
mod timeline_renderer_command_bridge;
mod timeline_renderer_commands;
mod timeline_renderer_platform_strategy;
pub mod timeline_renderer_supervisor;
pub mod timeline_renderer_window_thread;

pub use desktop_smoke::{
    graph_renderer_lifecycle_smoke_report_json, smoke_report_json,
    timeline_renderer_lifecycle_smoke_report_json,
};

use bevy_graph_host::DesktopBibleGraphRendererOwner;
use desktop_events::DesktopEventBridgeOwner;
use eidetic_server::state::AppState;
use graph_renderer_projection::GraphRendererProjectionOwner;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            let graph_owner = DesktopBibleGraphRendererOwner::start().unwrap_or_else(|error| {
                DesktopBibleGraphRendererOwner::unavailable(format!(
                    "failed to start Bevy bible graph renderer owner: {error:?}"
                ))
            });
            let graph_settings =
                graph_renderer_commands::load_graph_renderer_text_editor_settings(app.handle())
                    .map_err(|error| {
                        format!("failed to load Bevy bible graph renderer settings: {error:?}")
                    })?;
            graph_owner
                .apply_text_editor_settings(graph_settings)
                .map_err(|error| {
                    format!("failed to apply Bevy bible graph renderer settings: {error:?}")
                })?;
            app.manage(graph_owner);
            app.manage(GraphRendererProjectionOwner::new(app_state.clone()));
            app.manage(
                bevy_timeline_host::DesktopTimelineRendererOwner::start().unwrap_or_else(|error| {
                    bevy_timeline_host::DesktopTimelineRendererOwner::unavailable(format!(
                        "failed to start Bevy timeline renderer owner: {error:?}"
                    ))
                }),
            );
            app.manage(DesktopEventBridgeOwner::spawn(
                app.handle().clone(),
                &app_state,
            ));
            app.manage(app_state);
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                if let Some(event_bridges) = window.try_state::<DesktopEventBridgeOwner>() {
                    event_bridges.stop();
                }
                if let Some(graph_owner) = window.try_state::<DesktopBibleGraphRendererOwner>() {
                    let _ = graph_owner.stop();
                }
                if let Some(timeline_owner) =
                    window.try_state::<bevy_timeline_host::DesktopTimelineRendererOwner>()
                {
                    let _ = timeline_owner.stop();
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
            graph_renderer_commands::graph_renderer_update_projection_request,
            graph_renderer_commands::graph_renderer_camera_command,
            graph_renderer_commands::graph_renderer_text_editor_settings,
            graph_renderer_commands::graph_renderer_text_editor_settings_load,
            graph_renderer_commands::graph_renderer_text_editor_settings_save,
            graph_renderer_commands::graph_renderer_visual_snapshot,
            timeline_renderer_commands::timeline_renderer_open,
            timeline_renderer_commands::timeline_renderer_focus,
            timeline_renderer_commands::timeline_renderer_status,
            timeline_renderer_commands::timeline_renderer_close,
            reference_commands::reference_list,
            reference_commands::reference_upload,
            reference_commands::reference_delete,
            commands::object_script_story::command_object_field,
            commands::object_script_story::command_script_block,
            commands::object_script_story::command_script_lock,
            commands::object_script_story::command_story_create,
            commands::object_script_story::command_story_update,
            commands::object_script_story::command_story_delete,
            commands::affect::command_affect_set,
            commands::affect::command_affect_proposal_create,
            commands::affect::command_affect_proposal_reject,
            commands::affect::command_affect_proposal_accept,
            commands::bible::command_bible_graph_node,
            commands::bible::command_bible_graph_connected_node,
            commands::bible::command_bible_graph_delete_node,
            commands::bible::command_bible_graph_node_name,
            commands::bible::command_bible_graph_node_text,
            commands::bible::command_bible_graph_field,
            commands::bible::command_bible_graph_edge,
            commands::bible::command_bible_graph_delete_edge,
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
            commands::timeline::command_timeline_create_child_from_parent,
            commands::timeline::command_timeline_node_range,
            commands::timeline::command_timeline_node_lock,
            commands::timeline::command_timeline_node_notes,
            commands::timeline::command_timeline_delete_node,
            commands::timeline::command_timeline_create_relationship,
            commands::timeline::command_timeline_delete_relationship,
            commands::timeline::command_timeline_apply_children,
            commands::timeline::command_timeline_split_node,
            commands::timeline::command_timeline_playhead,
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
            projections::affect::projection_affect,
            projections::affect::projection_affect_proposals,
            projections::timeline::projection_timeline_render,
            projections::timeline::projection_selected_node
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Eidetic desktop application");
}
