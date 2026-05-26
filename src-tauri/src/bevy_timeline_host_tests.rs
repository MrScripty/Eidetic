use eidetic_core::contracts::{TimelineRenderClip, TimelineRenderProjection, TimelineRenderTrack};
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;

use crate::timeline_renderer_platform_strategy::{
    TimelineRendererPlatformStrategy, TimelineRendererRunnerStartupPlan,
};
use crate::timeline_renderer_window_thread::TimelineRendererWindowThreadHandle;

use super::*;

#[test]
fn timeline_host_ingests_projection_and_reports_scene_counts() {
    let node_id = NodeId::new();
    let mut host = DesktopTimelineHost::new();

    let status = host.set_projection(projection_with_node(node_id)).unwrap();

    assert_eq!(
        status.renderer_window_kind,
        DesktopRendererWindowKind::Timeline
    );
    assert!(status.running);
    assert!(status.renderer_window_open);
    assert!(status.renderer_scene_ready);
    assert_eq!(
        status.renderer_window_lifecycle,
        DesktopRendererWindowLifecycle::SceneReadyPendingNativeRunner
    );
    assert_eq!(
        status.renderer_runner_lifecycle,
        DesktopRendererRunnerLifecycle::Closed
    );
    assert_eq!(
        status.renderer_runner_threading_model,
        expected_threading_model()
    );
    assert_eq!(
        status.renderer_window_strategy,
        DesktopRendererWindowStrategy::BevyWinitFloatingWindow
    );
    assert_eq!(
        status.renderer_window_capability,
        expected_strategy_status().capability
    );
    assert_eq!(
        status.renderer_window_capability_reason,
        expected_strategy_status().capability_reason
    );
    assert_eq!(
        status.renderer_window_verified_support,
        expected_strategy_status().verified_support
    );
    assert_eq!(
        status.renderer_window_visible_supported,
        expected_strategy_status().visible_window_supported
    );
    assert!(!status.renderer_window_visible);
    assert!(!status.renderer_window_ready);
    assert!(!status.renderer_window_focus_supported);
    assert_eq!(
        status.renderer_window_message,
        "timeline renderer scene is ready; native window is not connected"
    );
    assert_eq!(status.track_count, 1);
    assert_eq!(status.clip_count, 1);
    assert_eq!(status.relationship_count, 0);
    assert_eq!(status.affect_overlay_count, 0);
    assert_eq!(status.queued_command_count, 0);
}

#[test]
fn timeline_host_drain_commands_is_empty_without_renderer_commands() {
    let mut host = DesktopTimelineHost::new();

    assert!(host.drain_commands().is_empty());
    assert!(!host.status().running);
}

#[test]
fn timeline_host_stop_clears_projection_state() {
    let node_id = NodeId::new();
    let mut host = DesktopTimelineHost::new();
    host.set_projection(projection_with_node(node_id)).unwrap();

    let status = host.stop();

    assert!(!status.running);
    assert!(!status.renderer_window_open);
    assert!(!status.renderer_scene_ready);
    assert_eq!(
        status.renderer_window_lifecycle,
        DesktopRendererWindowLifecycle::Closed
    );
    assert_eq!(
        status.renderer_runner_lifecycle,
        DesktopRendererRunnerLifecycle::Closed
    );
    assert_eq!(
        status.renderer_window_capability,
        expected_strategy_status().capability
    );
    assert!(!status.renderer_window_visible);
    assert!(!status.renderer_window_ready);
    assert!(!status.renderer_window_focus_supported);
    assert_eq!(
        status.renderer_window_message,
        "floating timeline renderer window is closed"
    );
    assert_eq!(status.track_count, 0);
    assert_eq!(status.clip_count, 0);
}

#[test]
fn timeline_host_open_renderer_starts_injected_native_window() {
    let node_id = NodeId::new();
    let mut host = DesktopTimelineHost::with_native_window(
        TimelineRendererSupervisor::for_strategy_with_window_thread_start(
            TimelineRendererPlatformStrategy::LinuxWorkerThreadVerified,
            injected_ready_window_thread,
        ),
    );

    let status = host.open_renderer(projection_with_node(node_id)).unwrap();
    let stopped = host.stop();

    assert!(status.running);
    assert_eq!(status.clip_count, 1);
    assert_eq!(
        status.renderer_runner_lifecycle,
        DesktopRendererRunnerLifecycle::Visible
    );
    assert!(status.renderer_window_visible);
    assert_eq!(
        status.renderer_window_lifecycle,
        DesktopRendererWindowLifecycle::Visible
    );
    assert!(!stopped.running);
    assert!(!stopped.renderer_window_visible);
}

#[test]
fn timeline_owner_uses_bounded_command_queue() {
    assert_eq!(TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY, 128);
    assert_eq!(TIMELINE_RENDERER_REPLY_TIMEOUT_MS, 2_000);
}

#[test]
fn timeline_owner_ingests_projection_on_renderer_thread() {
    let node_id = NodeId::new();
    let owner = DesktopTimelineRendererOwner::start().unwrap();

    let status = owner.set_projection(projection_with_node(node_id)).unwrap();
    let stopped = owner.stop().unwrap();

    assert!(status.running);
    assert_eq!(status.clip_count, 1);
    assert!(!stopped.running);
}

#[test]
fn timeline_owner_focus_reports_current_lifecycle_without_starting_renderer() {
    let owner = DesktopTimelineRendererOwner::start().unwrap();

    let status = owner.focus_renderer().unwrap();
    let stopped = owner.stop().unwrap();

    assert!(!status.running);
    assert_eq!(
        status.renderer_window_lifecycle,
        DesktopRendererWindowLifecycle::Closed
    );
    assert_eq!(
        status.renderer_runner_lifecycle,
        DesktopRendererRunnerLifecycle::Closed
    );
    assert_eq!(
        status.renderer_window_capability,
        expected_strategy_status().capability
    );
    assert!(!status.renderer_window_focus_supported);
    assert!(!stopped.running);
}

#[test]
fn timeline_owner_unavailable_reports_status_without_renderer_thread() {
    let owner = DesktopTimelineRendererOwner::unavailable("native runner disabled".to_string());

    let status = owner.status().unwrap();
    let commands = owner.drain_commands().unwrap();
    let stopped = owner.stop().unwrap();

    assert!(!status.running);
    assert!(!status.renderer_window_open);
    assert_eq!(
        status.renderer_window_lifecycle,
        DesktopRendererWindowLifecycle::Closed
    );
    assert_eq!(
        status.renderer_window_message,
        "timeline renderer native window is unavailable"
    );
    assert_eq!(status.last_error.as_deref(), Some("native runner disabled"));
    assert!(commands.is_empty());
    assert_eq!(stopped, status);
}

fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
    let track_id = TrackId::new();
    TimelineRenderProjection {
        total_duration_ms: 10_000,
        selected_node_id: None,
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
            name: "Beach argument".to_string(),
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

fn expected_strategy_status() -> DesktopRendererWindowStrategyStatus {
    TimelineRendererPlatformStrategy::current().status()
}

fn expected_threading_model() -> DesktopRendererThreadingModel {
    match TimelineRendererPlatformStrategy::current().runner_startup_plan() {
        TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate {
            threading_model, ..
        }
        | TimelineRendererRunnerStartupPlan::PendingOnly { threading_model } => threading_model,
    }
}

fn injected_ready_window_thread(
    config: eidetic_bevy_timeline::TimelineNativeWindowRunnerConfig,
) -> std::io::Result<TimelineRendererWindowThreadHandle> {
    TimelineRendererWindowThreadHandle::start_with(config, |_config, control| {
        control.mark_ready();
        control.mark_visible(true);
        while !control.close_requested() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    })
}
