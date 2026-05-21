mod commands;
mod error;
mod health;
mod project_commands;
mod projections;

use eidetic_server::state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health::desktop_health,
            project_commands::project_create,
            project_commands::project_get,
            project_commands::project_update,
            project_commands::project_save,
            project_commands::project_load,
            project_commands::project_list,
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
            commands::timeline::command_timeline_delete_relationship,
            commands::timeline::command_timeline_split_node,
            projections::story_script::projection_object_field,
            projections::story_script::projection_script_document,
            projections::bible::projection_bible_graph_node,
            projections::bible::projection_bible_graph_nodes,
            projections::bible::projection_bible_graph_schemas,
            projections::bible::projection_bible_render_graph,
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
