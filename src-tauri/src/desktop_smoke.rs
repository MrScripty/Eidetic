use std::time::Duration;

use eidetic_core::contracts::{TimelineRenderClip, TimelineRenderProjection, TimelineRenderTrack};
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;
use eidetic_server::state::AppState;
use serde::Serialize;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::bevy_timeline_host::{DesktopTimelineRendererOwner, TimelineHostStatus};

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

#[derive(Serialize)]
pub struct TimelineRendererLifecycleSmokeReport {
    status: &'static str,
    boundary: &'static str,
    renderer: &'static str,
    open: TimelineHostStatus,
    status_after_open: TimelineHostStatus,
    focus: TimelineHostStatus,
    close: TimelineHostStatus,
    reopen: TimelineHostStatus,
    project_close: TimelineHostStatus,
    app_shutdown: TimelineHostStatus,
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
    tauri::async_runtime::block_on(app_state.shutdown_tasks_async());
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

pub fn timeline_renderer_lifecycle_smoke_report()
-> Result<TimelineRendererLifecycleSmokeReport, String> {
    timeline_renderer_lifecycle_smoke_report_with_owner(
        "native_bevy",
        DesktopTimelineRendererOwner::start()
            .map_err(|error| format!("failed to start timeline renderer owner: {error:?}"))?,
    )
}

pub fn timeline_renderer_lifecycle_smoke_report_json() -> Result<String, String> {
    serde_json::to_string(&timeline_renderer_lifecycle_smoke_report()?).map_err(|error| {
        format!("failed to serialize timeline renderer lifecycle smoke report: {error}")
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

fn timeline_renderer_lifecycle_smoke_report_with_owner(
    renderer: &'static str,
    owner: DesktopTimelineRendererOwner,
) -> Result<TimelineRendererLifecycleSmokeReport, String> {
    owner
        .open_renderer(timeline_renderer_smoke_projection())
        .map_err(|error| format!("timeline renderer smoke open failed: {error:?}"))?;
    let open = wait_for_timeline_renderer_ready(&owner, "open")?;
    let status_after_open = owner
        .status()
        .map_err(|error| format!("timeline renderer smoke status failed: {error:?}"))?;
    let focus = owner
        .focus_renderer()
        .map_err(|error| format!("timeline renderer smoke focus failed: {error:?}"))?;
    let close = owner
        .close_renderer()
        .map_err(|error| format!("timeline renderer smoke close failed: {error:?}"))?;
    owner
        .open_renderer(timeline_renderer_smoke_projection())
        .map_err(|error| format!("timeline renderer smoke reopen failed: {error:?}"))?;
    let reopen = wait_for_timeline_renderer_ready(&owner, "reopen")?;
    let project_close = owner
        .close_renderer()
        .map_err(|error| format!("timeline renderer smoke project close failed: {error:?}"))?;
    let app_shutdown = owner
        .stop()
        .map_err(|error| format!("timeline renderer smoke app shutdown failed: {error:?}"))?;

    let report = TimelineRendererLifecycleSmokeReport {
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
    validate_timeline_renderer_lifecycle_smoke_report(&report)?;
    Ok(report)
}

fn validate_timeline_renderer_lifecycle_smoke_report(
    report: &TimelineRendererLifecycleSmokeReport,
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
                "timeline renderer lifecycle smoke {label} expected open={expected_open} but saw open={}",
                status.renderer_window_open
            ));
        }
        if status.last_error.is_some() {
            return Err(format!(
                "timeline renderer lifecycle smoke {label} reported error: {}",
                status.last_error.as_deref().unwrap_or("unknown")
            ));
        }
        if expected_open && !status.renderer_window_ready {
            return Err(format!(
                "timeline renderer lifecycle smoke {label} expected a ready native window"
            ));
        }
    }

    Ok(())
}

fn wait_for_timeline_renderer_ready(
    owner: &DesktopTimelineRendererOwner,
    label: &str,
) -> Result<TimelineHostStatus, String> {
    for _ in 0..2_000 {
        let status = owner
            .status()
            .map_err(|error| format!("timeline renderer smoke {label} status failed: {error:?}"))?;
        if status.renderer_window_ready || status.last_error.is_some() {
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    owner
        .status()
        .map_err(|error| format!("timeline renderer smoke {label} status failed: {error:?}"))
}

fn timeline_renderer_smoke_projection() -> TimelineRenderProjection {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    TimelineRenderProjection {
        total_duration_ms: 10_000,
        selected_node_id: Some(node_id),
        structure_segments: Vec::new(),
        tracks: vec![TimelineRenderTrack {
            track_id,
            level: StoryLevel::Scene,
            label: "Scenes".to_string(),
            sort_order: 30,
            collapsed: false,
        }],
        clips: vec![TimelineRenderClip {
            node_id,
            parent_id: None,
            track_id,
            level: StoryLevel::Scene,
            name: "Timeline smoke scene".to_string(),
            start_ms: 1_000,
            end_ms: 4_000,
            sort_order: 10,
            locked: false,
            content_status: ContentStatus::NotesOnly,
            beat_type: None,
            arc_ids: Vec::new(),
        }],
        relationships: Vec::new(),
        gaps: Vec::new(),
        affect_overlays: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bevy_graph_host::NativeRendererWindowThreadHandle;
    use crate::timeline_renderer_window_thread::TimelineRendererWindowThreadHandle;
    use eidetic_bevy_bible_graph::BibleGraphNativeWindowRunnerConfig;
    use eidetic_bevy_timeline::TimelineNativeWindowRunnerConfig;

    #[test]
    fn smoke_report_initializes_backend_runtime() {
        let report = smoke_report();

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

    #[test]
    fn timeline_renderer_lifecycle_smoke_exercises_managed_owner() {
        let owner = DesktopTimelineRendererOwner::start_with_native_window_thread_start(
            start_test_timeline_window_thread,
        )
        .unwrap();

        let report =
            timeline_renderer_lifecycle_smoke_report_with_owner("test_native_bevy", owner).unwrap();

        assert_eq!(report.status, "ok");
        assert_eq!(report.boundary, "tauri_managed_backend");
        assert_eq!(report.renderer, "test_native_bevy");
        assert!(report.open.renderer_window_open);
        assert_eq!(report.open.clip_count, 1);
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

    fn start_test_timeline_window_thread(
        config: TimelineNativeWindowRunnerConfig,
    ) -> std::io::Result<TimelineRendererWindowThreadHandle> {
        TimelineRendererWindowThreadHandle::start_with(config, |_config, control| {
            control.mark_ready();
            while !control.close_requested() {
                std::thread::sleep(Duration::from_millis(1));
            }
        })
    }
}
