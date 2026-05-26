mod ai_commands;
pub mod bevy_graph_host;
mod commands;
mod desktop_events;
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

use bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use desktop_events::DesktopEventBridgeOwner;
use eidetic_server::state::AppState;
use graph_renderer_projection::GraphRendererProjectionOwner;
use serde::Serialize;
use std::time::Duration;
use tauri::Manager;

#[derive(Serialize)]
pub struct DesktopSmokeReport {
    status: &'static str,
    boundary: &'static str,
    backend_runtime: &'static str,
    active_backend_tasks: usize,
    model_library_configured: bool,
}

#[derive(Serialize)]
pub struct GraphRendererLifecycleSmokeReport {
    status: &'static str,
    boundary: &'static str,
    renderer: &'static str,
    open: BibleGraphHostStatus,
    status_after_open: BibleGraphHostStatus,
    focus: BibleGraphHostStatus,
    close: BibleGraphHostStatus,
    reopen: BibleGraphHostStatus,
    project_close: BibleGraphHostStatus,
    app_shutdown: BibleGraphHostStatus,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            app.manage(
                DesktopBibleGraphRendererOwner::start().unwrap_or_else(|error| {
                    DesktopBibleGraphRendererOwner::unavailable(format!(
                        "failed to start Bevy bible graph renderer owner: {error:?}"
                    ))
                }),
            );
            app.manage(GraphRendererProjectionOwner::new(app_state.clone()));
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
            commands::affect::command_affect_set,
            commands::bible::command_bible_graph_node,
            commands::bible::command_bible_graph_delete_node,
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
            projections::affect::projection_affect,
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

pub fn graph_renderer_lifecycle_smoke_report() -> Result<GraphRendererLifecycleSmokeReport, String>
{
    graph_renderer_lifecycle_smoke_report_with_owner(
        "native_bevy",
        DesktopBibleGraphRendererOwner::start()
            .map_err(|error| format!("failed to start graph renderer owner: {error:?}"))?,
    )
}

pub fn graph_renderer_lifecycle_smoke_report_json() -> Result<String, String> {
    serde_json::to_string(&graph_renderer_lifecycle_smoke_report()?).map_err(|error| {
        format!("failed to serialize graph renderer lifecycle smoke report: {error}")
    })
}

fn graph_renderer_lifecycle_smoke_report_with_owner(
    renderer: &'static str,
    owner: DesktopBibleGraphRendererOwner,
) -> Result<GraphRendererLifecycleSmokeReport, String> {
    owner
        .start_renderer()
        .map_err(|error| format!("graph renderer smoke open failed: {error:?}"))?;
    let open = wait_for_graph_renderer_ready(&owner, "open")?;
    let status_after_open = owner
        .status()
        .map_err(|error| format!("graph renderer smoke status failed: {error:?}"))?;
    let focus = owner
        .focus_renderer()
        .map_err(|error| format!("graph renderer smoke focus failed: {error:?}"))?;
    let close = owner
        .close_renderer()
        .map_err(|error| format!("graph renderer smoke close failed: {error:?}"))?;
    owner
        .start_renderer()
        .map_err(|error| format!("graph renderer smoke reopen failed: {error:?}"))?;
    let reopen = wait_for_graph_renderer_ready(&owner, "reopen")?;
    let project_close = owner
        .close_renderer()
        .map_err(|error| format!("graph renderer smoke project close failed: {error:?}"))?;
    let app_shutdown = owner
        .stop()
        .map_err(|error| format!("graph renderer smoke app shutdown failed: {error:?}"))?;

    let report = GraphRendererLifecycleSmokeReport {
        status: "ok",
        boundary: "tauri_managed_backend",
        renderer,
        open,
        status_after_open,
        focus,
        close,
        reopen,
        project_close,
        app_shutdown,
    };
    validate_graph_renderer_lifecycle_smoke_report(&report)?;
    Ok(report)
}

fn validate_graph_renderer_lifecycle_smoke_report(
    report: &GraphRendererLifecycleSmokeReport,
) -> Result<(), String> {
    let expectations = [
        ("open", &report.open, true),
        ("status_after_open", &report.status_after_open, true),
        ("focus", &report.focus, true),
        ("close", &report.close, false),
        ("reopen", &report.reopen, true),
        ("project_close", &report.project_close, false),
        ("app_shutdown", &report.app_shutdown, false),
    ];

    for (label, status, expected_open) in expectations {
        if status.renderer_window_open != expected_open {
            return Err(format!(
                "graph renderer lifecycle smoke {label} expected open={expected_open} but saw open={}",
                status.renderer_window_open
            ));
        }
        if status.last_error.is_some() {
            return Err(format!(
                "graph renderer lifecycle smoke {label} reported error: {}",
                status.last_error.as_deref().unwrap_or("unknown")
            ));
        }
        if expected_open && !status.renderer_window_ready {
            return Err(format!(
                "graph renderer lifecycle smoke {label} expected a ready native window"
            ));
        }
    }

    Ok(())
}

fn wait_for_graph_renderer_ready(
    owner: &DesktopBibleGraphRendererOwner,
    label: &str,
) -> Result<BibleGraphHostStatus, String> {
    for _ in 0..2_000 {
        let status = owner
            .status()
            .map_err(|error| format!("graph renderer smoke {label} status failed: {error:?}"))?;
        if status.renderer_window_ready || status.last_error.is_some() {
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    owner
        .status()
        .map_err(|error| format!("graph renderer smoke {label} status failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bevy_graph_host::NativeRendererWindowThreadHandle;
    use eidetic_bevy_bible_graph::BibleGraphNativeWindowRunnerConfig;

    #[test]
    fn smoke_report_initializes_backend_runtime() {
        let report = super::smoke_report();

        assert_eq!(report.status, "ok");
        assert_eq!(report.boundary, "tauri");
        assert_eq!(report.backend_runtime, "initialized");
        assert!(report.active_backend_tasks >= 2);
    }

    #[test]
    fn graph_renderer_lifecycle_smoke_exercises_managed_owner() {
        let owner = DesktopBibleGraphRendererOwner::start_with_native_window_thread_start(
            start_test_window_thread,
        )
        .unwrap();

        let report =
            graph_renderer_lifecycle_smoke_report_with_owner("test_native_bevy", owner).unwrap();

        assert_eq!(report.status, "ok");
        assert_eq!(report.boundary, "tauri_managed_backend");
        assert_eq!(report.renderer, "test_native_bevy");
        assert!(report.open.renderer_window_open);
        assert!(report.status_after_open.renderer_window_open);
        assert!(report.focus.renderer_window_open);
        assert!(!report.close.renderer_window_open);
        assert!(report.reopen.renderer_window_open);
        assert!(!report.project_close.renderer_window_open);
        assert!(!report.app_shutdown.renderer_window_open);
    }

    fn start_test_window_thread(
        config: BibleGraphNativeWindowRunnerConfig,
    ) -> std::io::Result<NativeRendererWindowThreadHandle> {
        NativeRendererWindowThreadHandle::start_with(config, |_config, control| {
            control.mark_ready();
            while !control.close_requested() {
                std::thread::sleep(Duration::from_millis(1));
            }
        })
    }
}
