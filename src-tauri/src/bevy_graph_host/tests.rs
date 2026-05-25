use eidetic_bevy_bible_graph::{
    BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, BibleGraphCameraCommand,
    BibleGraphNativeWindowRunnerConfig, BibleGraphRendererCommand,
};
use eidetic_core::contracts::{
    BibleGraphEdgeKind, BibleGraphNodeId, BibleRenderGraphEdge, BibleRenderGraphInfluence,
    BibleRenderGraphNeighborhood, BibleRenderGraphNode, BibleRenderGraphPosition,
    ContextInfluenceId, ContextInfluenceKind, ContextInfluenceProvenance,
};
use eidetic_core::timeline::node::StoryLevel;
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

use crate::renderer_window::DesktopRendererWindowKind;

use super::{
    BibleGraphHostError, BibleGraphHostStatus, BibleGraphRendererWindowCapability,
    BibleGraphRendererWindowCapabilityReason, BibleGraphRendererWindowLifecycle,
    BibleGraphRendererWindowPlatform, BibleGraphRendererWindowStrategy,
    BibleGraphRendererWindowStrategyStatus, DesktopBibleGraphHost, DesktopBibleGraphRendererOwner,
    GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, GRAPH_RENDERER_REPLY_TIMEOUT_MS,
    NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY, NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS,
    NativeRendererPlatformStrategy, NativeRendererRunner, NativeRendererRunnerHandle,
    NativeRendererRunnerLifecycle, NativeRendererRunnerStartupPlan, NativeRendererSupervisor,
    NativeRendererSupervisorLifecycle, NativeRendererThreadingModel,
    NativeRendererWindowThreadHandle, NativeRendererWindowThreadResult,
};

#[test]
fn owner_uses_bounded_command_queue() {
    assert_eq!(GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, 128);
    assert_eq!(GRAPH_RENDERER_REPLY_TIMEOUT_MS, 2_000);
    assert_eq!(
        GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY,
        BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY
    );
}

#[test]
fn native_renderer_runner_uses_bounded_command_queue() {
    assert_eq!(NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY, 16);
    assert_eq!(NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS, 2_000);
}

#[test]
fn renderer_window_strategy_reports_typed_capability() {
    let status = BibleGraphRendererWindowStrategyStatus::current();

    assert_eq!(
        status.strategy,
        BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow
    );
    assert_eq!(status.platform, BibleGraphRendererWindowPlatform::current());
    assert_eq!(status.capability, expected_capability());
    assert_eq!(status.capability_reason, expected_pending_reason());
    assert_eq!(status.verified_support, expected_verified_support());
    assert_eq!(
        status.visible_window_supported,
        expected_visible_window_supported()
    );
}

#[test]
fn native_renderer_platform_strategy_reports_current_platform_status() {
    let strategy = NativeRendererPlatformStrategy::current();
    let status = strategy.status();

    assert_eq!(status.platform, BibleGraphRendererWindowPlatform::current());
    assert_eq!(
        status.strategy,
        BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow
    );
    assert_eq!(status.capability, expected_capability());
    assert_eq!(status.capability_reason, expected_pending_reason());
    assert_eq!(status.verified_support, expected_verified_support());
    assert_eq!(
        status.visible_window_supported,
        expected_visible_window_supported()
    );
}

#[test]
fn renderer_window_capability_owns_support_flags() {
    assert!(!BibleGraphRendererWindowCapability::PendingNativeRunner.verified_support());
    assert!(!BibleGraphRendererWindowCapability::PlatformUnproven.verified_support());
    assert!(!BibleGraphRendererWindowCapability::PlatformUnsupported.verified_support());
    assert!(!BibleGraphRendererWindowCapability::RunnerError.verified_support());
    assert!(BibleGraphRendererWindowCapability::VerifiedSupport.verified_support());

    assert!(!BibleGraphRendererWindowCapability::PendingNativeRunner.visible_window_supported());
    assert!(!BibleGraphRendererWindowCapability::PlatformUnproven.visible_window_supported());
    assert!(!BibleGraphRendererWindowCapability::PlatformUnsupported.visible_window_supported());
    assert!(!BibleGraphRendererWindowCapability::RunnerError.visible_window_supported());
    assert!(BibleGraphRendererWindowCapability::VerifiedSupport.visible_window_supported());
}

#[test]
fn native_renderer_platform_strategy_models_threading_requirements() {
    assert_eq!(
        NativeRendererPlatformStrategy::LinuxWorkerThreadVerified.threading_model(),
        NativeRendererThreadingModel::WorkerThread
    );
    assert!(
        NativeRendererPlatformStrategy::LinuxWorkerThreadVerified
            .can_attempt_minimal_window_proof()
    );
    assert_eq!(
        NativeRendererPlatformStrategy::WindowsWorkerThreadUnproven.threading_model(),
        NativeRendererThreadingModel::WorkerThread
    );
    assert!(
        NativeRendererPlatformStrategy::WindowsWorkerThreadUnproven
            .can_attempt_minimal_window_proof()
    );
    assert_eq!(
        NativeRendererPlatformStrategy::MacosMainThreadUnproven.threading_model(),
        NativeRendererThreadingModel::MainThread
    );
    assert!(
        NativeRendererPlatformStrategy::MacosMainThreadUnproven.can_attempt_minimal_window_proof()
    );
    assert_eq!(
        NativeRendererPlatformStrategy::UnsupportedPlatform.threading_model(),
        NativeRendererThreadingModel::Unsupported
    );
    assert!(
        !NativeRendererPlatformStrategy::UnsupportedPlatform.can_attempt_minimal_window_proof()
    );
}

#[test]
fn native_renderer_platform_strategy_builds_minimal_window_proof_config() {
    let linux_config = NativeRendererPlatformStrategy::LinuxWorkerThreadVerified
        .minimal_window_runner_config()
        .unwrap();
    let windows_config = NativeRendererPlatformStrategy::WindowsWorkerThreadUnproven
        .minimal_window_runner_config()
        .unwrap();
    let macos_config = NativeRendererPlatformStrategy::MacosMainThreadUnproven
        .minimal_window_runner_config()
        .unwrap();

    assert!(linux_config.run_on_any_thread);
    assert!(windows_config.run_on_any_thread);
    assert!(!macos_config.run_on_any_thread);
    assert_eq!(linux_config.title, "Eidetic Bible Graph");
    assert_eq!(linux_config.width_px, 1280);
    assert_eq!(linux_config.height_px, 720);
    assert!(
        NativeRendererPlatformStrategy::UnsupportedPlatform
            .minimal_window_runner_config()
            .is_none()
    );
}

#[test]
fn native_renderer_platform_strategy_builds_startup_plan() {
    let linux_plan =
        NativeRendererPlatformStrategy::LinuxWorkerThreadVerified.runner_startup_plan();
    let macos_plan = NativeRendererPlatformStrategy::MacosMainThreadUnproven.runner_startup_plan();
    let unsupported_plan =
        NativeRendererPlatformStrategy::UnsupportedPlatform.runner_startup_plan();

    assert!(matches!(
        linux_plan,
        NativeRendererRunnerStartupPlan::MinimalWindowProofCandidate {
            threading_model: NativeRendererThreadingModel::WorkerThread,
            ..
        }
    ));
    assert!(matches!(
        macos_plan,
        NativeRendererRunnerStartupPlan::MinimalWindowProofCandidate {
            threading_model: NativeRendererThreadingModel::MainThread,
            ..
        }
    ));
    assert_eq!(
        unsupported_plan,
        NativeRendererRunnerStartupPlan::PendingOnly {
            threading_model: NativeRendererThreadingModel::Unsupported
        }
    );
}

#[test]
fn native_renderer_supervisor_keeps_unsupported_platform_closed() {
    let mut runner =
        NativeRendererSupervisor::for_strategy(NativeRendererPlatformStrategy::UnsupportedPlatform);

    let initial = runner.status();

    assert_eq!(
        initial.strategy,
        BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow
    );
    assert_eq!(
        initial.platform,
        BibleGraphRendererWindowPlatform::Unsupported
    );
    assert_eq!(
        initial.capability,
        BibleGraphRendererWindowCapability::PlatformUnsupported
    );
    assert_eq!(
        initial.threading_model,
        NativeRendererThreadingModel::Unsupported
    );
    assert_eq!(initial.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        initial.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::NotStarted
    );
    assert_eq!(
        initial.capability_reason,
        BibleGraphRendererWindowCapabilityReason::PlatformUnsupported
    );
    assert!(!initial.verified_support);
    assert!(!initial.visible_window_supported);
    assert!(!initial.window_visible);
    assert!(!initial.window_ready);
    assert!(!initial.focus_supported);
    assert_eq!(initial.last_error, None);

    let opened = runner.open();

    let focused = runner.focus();
    let closed = runner.close();

    assert_eq!(opened.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        opened.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!opened.window_visible);
    assert!(!opened.window_ready);
    assert!(!opened.focus_supported);
    assert_eq!(opened.last_error, None);
    assert_eq!(focused.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        focused.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!focused.focus_supported);
    assert_eq!(focused.last_error, None);
    assert_eq!(closed.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        closed.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!closed.window_visible);
    assert_eq!(closed.last_error, None);
}

#[test]
fn native_renderer_supervisor_starts_injected_window_thread() {
    let mut runner = NativeRendererSupervisor::for_strategy_with_window_thread_start(
        NativeRendererPlatformStrategy::LinuxWorkerThreadVerified,
        start_test_window_thread,
    );

    let opened = runner.open();
    let focused = wait_for_supervisor_ready(&mut runner);
    let closed = runner.close();

    assert_eq!(opened.lifecycle, NativeRendererRunnerLifecycle::Visible);
    assert_eq!(
        opened.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Running
    );
    assert_eq!(
        opened.capability,
        BibleGraphRendererWindowCapability::VerifiedSupport
    );
    assert!(opened.verified_support);
    assert!(opened.visible_window_supported);
    assert!(opened.window_visible);
    assert!(!opened.window_ready);
    assert_eq!(opened.last_error, None);
    assert_eq!(focused.lifecycle, NativeRendererRunnerLifecycle::Visible);
    assert_eq!(
        focused.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Running
    );
    assert!(focused.window_visible);
    assert!(focused.window_ready);
    assert!(!focused.focus_supported);
    assert_eq!(focused.last_error, None);
    assert_eq!(closed.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        closed.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!closed.window_visible);
    assert_eq!(closed.last_error, None);
}

#[test]
fn native_renderer_runner_handle_routes_pending_commands_through_boundary() {
    let mut runner = NativeRendererRunnerHandle::start_for_strategy(
        NativeRendererPlatformStrategy::UnsupportedPlatform,
    )
    .unwrap();

    let opened = runner.open();
    let focused = runner.focus();
    let closed = runner.close();

    assert_eq!(
        opened.strategy,
        BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow
    );
    assert_eq!(
        opened.platform,
        BibleGraphRendererWindowPlatform::Unsupported
    );
    assert_eq!(opened.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        opened.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert_eq!(
        opened.capability,
        BibleGraphRendererWindowCapability::PlatformUnsupported
    );
    assert_eq!(
        opened.threading_model,
        NativeRendererThreadingModel::Unsupported
    );
    assert_eq!(
        opened.capability_reason,
        BibleGraphRendererWindowCapabilityReason::PlatformUnsupported
    );
    assert!(!opened.verified_support);
    assert!(!opened.visible_window_supported);
    assert!(!opened.window_visible);
    assert!(!opened.window_ready);
    assert_eq!(opened.last_error, None);
    assert_eq!(focused.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        focused.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!focused.focus_supported);
    assert_eq!(focused.last_error, None);
    assert_eq!(closed.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        closed.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!closed.window_visible);
    assert_eq!(closed.last_error, None);
}

#[test]
fn native_renderer_runner_handle_starts_from_explicit_platform_strategy() {
    let runner = NativeRendererRunnerHandle::start_for_strategy(
        NativeRendererPlatformStrategy::UnsupportedPlatform,
    )
    .unwrap();

    let status = runner.status();

    assert_eq!(
        status.platform,
        BibleGraphRendererWindowPlatform::Unsupported
    );
    assert_eq!(
        status.capability_reason,
        BibleGraphRendererWindowCapabilityReason::PlatformUnsupported
    );
    assert_eq!(
        status.capability,
        BibleGraphRendererWindowCapability::PlatformUnsupported
    );
    assert_eq!(status.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        status.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::NotStarted
    );
    assert!(!status.verified_support);
    assert!(!status.visible_window_supported);
}

#[test]
fn native_renderer_runner_handle_stops_with_bounded_reply() {
    let mut runner = NativeRendererRunnerHandle::start_for_strategy(
        NativeRendererPlatformStrategy::UnsupportedPlatform,
    )
    .unwrap();
    runner.open();

    let stopped = runner.stop();
    let after_stop = runner.status();

    assert_eq!(stopped.lifecycle, NativeRendererRunnerLifecycle::Closed);
    assert_eq!(
        stopped.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Closed
    );
    assert!(!stopped.window_visible);
    assert_eq!(stopped.last_error, None);
    assert_eq!(
        after_stop.capability_reason,
        BibleGraphRendererWindowCapabilityReason::RunnerError
    );
    assert_eq!(
        after_stop.capability,
        BibleGraphRendererWindowCapability::RunnerError
    );
    assert_eq!(
        after_stop.supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Failed
    );
    assert!(after_stop.last_error.is_some());
}

#[test]
fn native_renderer_window_thread_reports_completion() {
    let mut window_thread = NativeRendererWindowThreadHandle::start_with(
        NativeRendererPlatformStrategy::LinuxWorkerThreadVerified
            .minimal_window_runner_config()
            .unwrap(),
        |_config, _control| {},
    )
    .unwrap();

    let status = window_thread.stop(Duration::from_millis(250));

    assert!(status.close_requested);
    assert!(!status.ready);
    assert!(!status.running);
    assert_eq!(
        status.result,
        Some(NativeRendererWindowThreadResult::Completed)
    );
}

#[test]
fn native_renderer_window_thread_can_request_bounded_close() {
    let (started_sender, started_receiver) = mpsc::channel();
    let mut window_thread = NativeRendererWindowThreadHandle::start_with(
        NativeRendererPlatformStrategy::LinuxWorkerThreadVerified
            .minimal_window_runner_config()
            .unwrap(),
        move |_config, control| {
            started_sender.send(()).unwrap();
            control.mark_ready();
            while !control.close_requested() {
                std::thread::sleep(Duration::from_millis(1));
            }
        },
    )
    .unwrap();
    started_receiver
        .recv_timeout(Duration::from_millis(250))
        .unwrap();

    let running = window_thread.status();
    let stopped = window_thread.stop(Duration::from_millis(250));

    assert!(running.running);
    assert!(running.ready);
    assert!(!running.close_requested);
    assert!(stopped.close_requested);
    assert!(stopped.ready);
    assert!(!stopped.running);
    assert_eq!(
        stopped.result,
        Some(NativeRendererWindowThreadResult::Completed)
    );
}

#[test]
fn renderer_window_lifecycle_is_derived_from_backend_state() {
    assert_eq!(
        BibleGraphRendererWindowLifecycle::from_state(false, false, false),
        BibleGraphRendererWindowLifecycle::Closed
    );
    assert_eq!(
        BibleGraphRendererWindowLifecycle::from_state(true, false, false),
        BibleGraphRendererWindowLifecycle::SceneStarting
    );
    assert_eq!(
        BibleGraphRendererWindowLifecycle::from_state(true, true, false),
        BibleGraphRendererWindowLifecycle::SceneReadyPendingNativeRunner
    );
    assert_eq!(
        BibleGraphRendererWindowLifecycle::from_state(true, true, true),
        BibleGraphRendererWindowLifecycle::Visible
    );
    assert_eq!(
        BibleGraphRendererWindowLifecycle::from_state(false, true, true),
        BibleGraphRendererWindowLifecycle::Closed
    );
    assert_eq!(
        BibleGraphRendererWindowLifecycle::from_state(true, false, true),
        BibleGraphRendererWindowLifecycle::SceneStarting
    );
}

#[test]
fn host_applies_projection_and_reports_scene_counts() {
    let mut host = test_bible_graph_host();

    let status = host.set_projection(sample_projection()).unwrap();

    assert_eq!(
        status.renderer_window_kind,
        DesktopRendererWindowKind::BibleGraph
    );
    assert!(status.running);
    assert!(status.renderer_window_open);
    assert!(status.renderer_scene_ready);
    assert!(status.renderer_window_visible);
    assert_eq!(
        status.renderer_window_strategy,
        BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow
    );
    assert_eq!(
        status.renderer_window_platform,
        BibleGraphRendererWindowPlatform::current()
    );
    assert_eq!(
        status.renderer_runner_lifecycle,
        NativeRendererRunnerLifecycle::Visible
    );
    assert_eq!(
        status.renderer_supervisor_lifecycle,
        NativeRendererSupervisorLifecycle::Running
    );
    assert_eq!(
        status.renderer_runner_threading_model,
        expected_threading_model()
    );
    assert_eq!(status.renderer_window_capability, expected_capability());
    assert_eq!(
        status.renderer_window_capability_reason,
        expected_pending_reason()
    );
    assert_eq!(
        status.renderer_window_lifecycle,
        BibleGraphRendererWindowLifecycle::Visible
    );
    assert_eq!(
        status.renderer_window_verified_support,
        expected_verified_support()
    );
    assert_eq!(
        status.renderer_window_visible_supported,
        expected_visible_window_supported()
    );
    assert!(!status.renderer_window_focus_supported);
    assert_eq!(
        status.renderer_window_message,
        expected_visible_renderer_window_message(status.renderer_window_ready)
    );
    assert_eq!(status.node_count, 2);
    assert_eq!(status.edge_count, 1);
    assert_eq!(status.native_visual_node_count, 0);
    assert_eq!(status.native_visual_edge_count, 0);
    assert_eq!(status.renderer_window_width_px, 0);
    assert_eq!(status.renderer_window_height_px, 0);
    assert_eq!(status.influence_count, 1);
    assert_eq!(status.last_error, None);
}

#[test]
fn host_projection_update_does_not_start_closed_renderer() {
    let mut host = test_bible_graph_host();

    let status = host.update_projection_if_open(sample_projection()).unwrap();

    assert!(!status.running);
    assert!(!status.renderer_window_open);
    assert_eq!(status.node_count, 0);
    assert_eq!(status.edge_count, 0);
}

#[test]
fn host_validates_renderer_commands_and_drains_them() {
    let mut host = test_bible_graph_host();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    let edge_id = projection.edges[0].edge_id.clone();
    let influence_id = projection.influences[0].influence_id;
    host.set_projection(projection).unwrap();

    host.select_node(node_id.clone()).unwrap();
    host.inspect_node(node_id.clone()).unwrap();
    host.select_edge(edge_id.clone()).unwrap();
    host.select_influence(influence_id).unwrap();

    assert_eq!(
        host.drain_commands(),
        vec![
            BibleGraphRendererCommand::SelectNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::InspectNode { node_id },
            BibleGraphRendererCommand::SelectEdge { edge_id },
            BibleGraphRendererCommand::SelectInfluence { influence_id },
        ]
    );
    assert!(host.drain_commands().is_empty());
}

#[test]
fn host_routes_validated_camera_commands_to_native_runner() {
    let mut host = test_bible_graph_host();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    host.set_projection(projection).unwrap();

    let status = host
        .apply_camera_command(BibleGraphCameraCommand::FrameNode {
            node_id: node_id.clone(),
        })
        .unwrap();

    assert!(status.renderer_window_open);
    assert!(status.running);
    assert_eq!(status.node_count, 2);
    assert_eq!(status.last_error, None);

    let missing_node_id = BibleGraphNodeId::new("node.missing").unwrap();
    let error = host
        .apply_camera_command(BibleGraphCameraCommand::FrameNode {
            node_id: missing_node_id,
        })
        .unwrap_err();

    assert!(matches!(error, BibleGraphHostError::Renderer(_)));
    assert!(host.status().last_error.is_some());
}

#[test]
fn host_exposes_renderer_visual_snapshot() {
    let mut host = test_bible_graph_host();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    let edge_id = projection.edges[0].edge_id.clone();
    host.set_projection(projection).unwrap();

    let snapshot = host.visual_snapshot().unwrap();

    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.edges.len(), 1);
    assert_eq!(snapshot.nodes[0].node_id, node_id);
    assert!(snapshot.nodes[0].highlighted);
    assert_eq!(snapshot.edges[0].edge_id, edge_id);
    assert!(snapshot.edges[0].highlighted);
}

#[test]
fn host_visual_snapshot_does_not_start_renderer_without_projection() {
    let mut host = test_bible_graph_host();

    let error = host.visual_snapshot().unwrap_err();
    let status = host.status();

    assert!(matches!(error, BibleGraphHostError::Renderer(_)));
    assert!(!status.running);
    assert!(!status.renderer_window_open);
    assert!(!status.renderer_scene_ready);
}

#[test]
fn host_records_renderer_errors_without_panicking() {
    let mut host = test_bible_graph_host();
    host.set_projection(sample_projection()).unwrap();
    let missing = BibleGraphNodeId::new("node.missing").unwrap();

    let error = host.select_node(missing).unwrap_err();

    assert!(matches!(error, BibleGraphHostError::Renderer(_)));
    assert!(host.status().last_error.is_some());
}

#[test]
fn host_stop_drops_renderer_state() {
    let mut host = test_bible_graph_host();
    host.set_projection(sample_projection()).unwrap();

    let status = host.stop();

    assert_eq!(
        status,
        BibleGraphHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::BibleGraph,
            running: false,
            renderer_window_open: false,
            renderer_scene_ready: false,
            renderer_window_visible: false,
            renderer_window_strategy: BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow,
            renderer_window_platform: BibleGraphRendererWindowPlatform::current(),
            renderer_runner_lifecycle: NativeRendererRunnerLifecycle::Closed,
            renderer_supervisor_lifecycle: NativeRendererSupervisorLifecycle::Closed,
            renderer_runner_threading_model: expected_threading_model(),
            renderer_window_capability: expected_capability(),
            renderer_window_capability_reason: expected_pending_reason(),
            renderer_window_lifecycle: BibleGraphRendererWindowLifecycle::Closed,
            renderer_window_ready: false,
            renderer_window_verified_support: expected_verified_support(),
            renderer_window_visible_supported: expected_visible_window_supported(),
            renderer_window_focus_supported: false,
            renderer_window_message: "floating graph renderer window is closed".to_string(),
            node_count: 0,
            edge_count: 0,
            native_visual_node_count: 0,
            native_visual_edge_count: 0,
            renderer_window_width_px: 0,
            renderer_window_height_px: 0,
            influence_count: 0,
            last_error: None,
        }
    );
}

#[test]
fn host_focus_routes_through_native_runner_status() {
    let mut host = test_bible_graph_host();
    host.start().unwrap();

    let status = host.focus();

    assert!(status.renderer_window_open);
    assert!(!status.renderer_window_focus_supported);
    assert!(status.renderer_window_visible);
}

#[test]
fn owner_runs_renderer_on_dedicated_thread() {
    let owner = test_bible_graph_owner();
    let projection = sample_projection();

    let status = owner.set_projection(projection).unwrap();

    assert_eq!(status.node_count, 2);
    assert_eq!(status.edge_count, 1);
    assert_eq!(status.native_visual_node_count, 0);
    assert_eq!(status.native_visual_edge_count, 0);
    assert_eq!(status.influence_count, 1);
    assert!(status.running);
    assert!(status.renderer_window_open);
    assert!(status.renderer_scene_ready);
    assert!(status.renderer_window_visible);
    assert_eq!(
        status.renderer_window_visible_supported,
        expected_visible_window_supported()
    );
    assert!(!status.renderer_window_focus_supported);
    owner.stop().unwrap();
}

#[test]
fn owner_can_start_renderer_before_projection_arrives() {
    let owner = test_bible_graph_owner();

    let status = owner.start_renderer().unwrap();

    assert!(status.running);
    assert!(status.renderer_window_open);
    assert!(status.renderer_scene_ready);
    assert!(status.renderer_window_visible);
    assert_eq!(status.node_count, 0);
    assert_eq!(status.edge_count, 0);
    assert_eq!(status.native_visual_node_count, 0);
    assert_eq!(status.native_visual_edge_count, 0);
    assert_eq!(status.influence_count, 0);
    owner.stop().unwrap();
}

#[test]
fn owner_focus_routes_to_renderer_thread() {
    let owner = test_bible_graph_owner();
    owner.start_renderer().unwrap();

    let status = owner.focus_renderer().unwrap();

    assert!(status.renderer_window_open);
    assert!(!status.renderer_window_focus_supported);
    assert!(status.renderer_window_visible);
    owner.stop().unwrap();
}

#[test]
fn owner_projection_update_does_not_start_closed_renderer() {
    let owner = test_bible_graph_owner();

    let status = owner
        .update_projection_if_open(sample_projection())
        .unwrap();

    assert!(!status.running);
    assert!(!status.renderer_window_open);
    assert_eq!(status.node_count, 0);
    assert_eq!(status.edge_count, 0);
    owner.stop().unwrap();
}

#[test]
fn owner_records_renderer_window_bounds_on_renderer_thread() {
    let owner = test_bible_graph_owner();

    let status = owner.set_renderer_window_bounds(1280, 720).unwrap();

    assert!(status.running);
    assert!(status.renderer_scene_ready);
    assert!(status.renderer_window_visible);
    assert_eq!(status.renderer_window_width_px, 1280);
    assert_eq!(status.renderer_window_height_px, 720);
    owner.stop().unwrap();
}

#[test]
fn owner_closes_renderer_without_stopping_owner_thread() {
    let owner = test_bible_graph_owner();
    owner.set_projection(sample_projection()).unwrap();

    let closed = owner.close_renderer().unwrap();
    let reopened = owner.start_renderer().unwrap();

    assert!(!closed.running);
    assert!(!closed.renderer_window_open);
    assert!(reopened.running);
    assert!(reopened.renderer_window_open);
    assert_eq!(reopened.node_count, 0);
    assert_eq!(reopened.edge_count, 0);
    owner.stop().unwrap();
}

#[test]
fn owner_rejects_empty_renderer_window_bounds_without_starting_renderer() {
    let owner = test_bible_graph_owner();

    let error = owner.set_renderer_window_bounds(0, 720).unwrap_err();
    let status = owner.status().unwrap();

    assert_eq!(
        error,
        BibleGraphHostError::InvalidRendererWindowBounds {
            width_px: 0,
            height_px: 720
        }
    );
    assert!(!status.running);
    assert!(!status.renderer_window_open);
    owner.stop().unwrap();
}

#[test]
fn owner_drains_validated_renderer_commands() {
    let owner = test_bible_graph_owner();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    let edge_id = projection.edges[0].edge_id.clone();
    let influence_id = projection.influences[0].influence_id;
    owner.set_projection(projection).unwrap();

    owner.select_node(node_id.clone()).unwrap();
    owner.inspect_node(node_id.clone()).unwrap();
    owner.select_edge(edge_id.clone()).unwrap();
    owner.select_influence(influence_id).unwrap();

    assert_eq!(
        owner.drain_commands().unwrap(),
        vec![
            BibleGraphRendererCommand::SelectNode {
                node_id: node_id.clone()
            },
            BibleGraphRendererCommand::InspectNode { node_id },
            BibleGraphRendererCommand::SelectEdge { edge_id },
            BibleGraphRendererCommand::SelectInfluence { influence_id },
        ]
    );
    assert!(owner.drain_commands().unwrap().is_empty());
    owner.stop().unwrap();
}

#[test]
fn owner_exposes_visual_snapshot_from_dedicated_thread() {
    let owner = test_bible_graph_owner();
    let projection = sample_projection();
    let node_id = projection.nodes[0].node_id.clone();
    owner.set_projection(projection).unwrap();

    let snapshot = owner.visual_snapshot().unwrap();

    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.nodes[0].node_id, node_id);
    assert!(snapshot.nodes[0].highlighted);
    owner.stop().unwrap();
}

#[test]
fn owner_reports_stopped_after_shutdown() {
    let owner = test_bible_graph_owner();
    owner.stop().unwrap();

    let error = owner.status().unwrap_err();

    assert_eq!(error, BibleGraphHostError::OwnerStopped);
}

#[test]
fn unavailable_owner_projects_typed_runner_error_status() {
    let owner = DesktopBibleGraphRendererOwner::unavailable("owner thread unavailable".to_string());

    let status = owner.status().unwrap();
    let opened = owner.start_renderer().unwrap();

    assert_eq!(status, opened);
    assert!(!status.running);
    assert!(!status.renderer_window_open);
    assert_eq!(
        status.renderer_window_capability,
        BibleGraphRendererWindowCapability::RunnerError
    );
    assert_eq!(
        status.renderer_window_capability_reason,
        BibleGraphRendererWindowCapabilityReason::RunnerError
    );
    assert!(!status.renderer_window_verified_support);
    assert!(!status.renderer_window_visible_supported);
    assert_eq!(
        status.last_error,
        Some("owner thread unavailable".to_string())
    );
    assert!(owner.drain_commands().unwrap().is_empty());
    assert!(matches!(
        owner.visual_snapshot(),
        Err(BibleGraphHostError::Renderer(_))
    ));
}

fn expected_pending_reason() -> BibleGraphRendererWindowCapabilityReason {
    match BibleGraphRendererWindowPlatform::current() {
        BibleGraphRendererWindowPlatform::Linux => {
            BibleGraphRendererWindowCapabilityReason::VerifiedSupport
        }
        BibleGraphRendererWindowPlatform::Macos | BibleGraphRendererWindowPlatform::Windows => {
            BibleGraphRendererWindowCapabilityReason::PlatformUnproven
        }
        BibleGraphRendererWindowPlatform::Unsupported => {
            BibleGraphRendererWindowCapabilityReason::PlatformUnsupported
        }
    }
}

fn expected_capability() -> BibleGraphRendererWindowCapability {
    match BibleGraphRendererWindowPlatform::current() {
        BibleGraphRendererWindowPlatform::Linux => {
            BibleGraphRendererWindowCapability::VerifiedSupport
        }
        BibleGraphRendererWindowPlatform::Macos | BibleGraphRendererWindowPlatform::Windows => {
            BibleGraphRendererWindowCapability::PlatformUnproven
        }
        BibleGraphRendererWindowPlatform::Unsupported => {
            BibleGraphRendererWindowCapability::PlatformUnsupported
        }
    }
}

fn expected_verified_support() -> bool {
    matches!(
        BibleGraphRendererWindowPlatform::current(),
        BibleGraphRendererWindowPlatform::Linux
    )
}

fn expected_visible_window_supported() -> bool {
    expected_verified_support()
}

fn expected_visible_renderer_window_message(window_ready: bool) -> &'static str {
    if window_ready {
        "graph renderer native window is ready"
    } else {
        "graph renderer native window is starting"
    }
}

fn expected_threading_model() -> NativeRendererThreadingModel {
    NativeRendererPlatformStrategy::current().threading_model()
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

fn wait_for_supervisor_ready(
    runner: &mut NativeRendererSupervisor,
) -> super::NativeRendererRunnerStatus {
    for _ in 0..100 {
        let status = runner.focus();
        if status.window_ready {
            return status;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    runner.focus()
}

fn test_bible_graph_host() -> DesktopBibleGraphHost {
    DesktopBibleGraphHost::new_with_native_window_thread_start(start_test_window_thread).unwrap()
}

fn test_bible_graph_owner() -> DesktopBibleGraphRendererOwner {
    DesktopBibleGraphRendererOwner::start_with_native_window_thread_start(start_test_window_thread)
        .unwrap()
}

fn sample_projection() -> eidetic_core::contracts::BibleRenderGraphProjection {
    let ada_id = BibleGraphNodeId::new("node.character.ada").unwrap();
    let beach_id = BibleGraphNodeId::new("node.location.beach").unwrap();
    let edge_id = eidetic_core::contracts::BibleGraphEdgeId::new("edge.ada.beach").unwrap();
    let influence_id = ContextInfluenceId(Uuid::from_u128(1));
    let timeline_node_id = eidetic_core::timeline::node::NodeId(Uuid::from_u128(2));

    eidetic_core::contracts::BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: Some(ada_id.clone()),
        selected_timeline_node_id: Some(timeline_node_id),
        active_timeline_ms: None,
        nodes: vec![
            BibleRenderGraphNode {
                node_id: ada_id.clone(),
                parent_id: None,
                schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("character").unwrap(),
                label: "Ada".to_string(),
                system_owned: false,
                sort_order: 0,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            BibleRenderGraphNode {
                node_id: beach_id.clone(),
                parent_id: None,
                schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("location").unwrap(),
                label: "Beach".to_string(),
                system_owned: false,
                sort_order: 1,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        ],
        edges: vec![BibleRenderGraphEdge {
            edge_id: edge_id.clone(),
            from_node_id: ada_id.clone(),
            to_node_id: beach_id.clone(),
            edge_kind: BibleGraphEdgeKind::LocatedIn,
            label: "located in".to_string(),
            directed: true,
            sort_order: 0,
        }],
        neighborhoods: vec![BibleRenderGraphNeighborhood {
            node_id: ada_id.clone(),
            connected_node_ids: vec![beach_id],
            edge_ids: vec![edge_id.clone()],
        }],
        influences: vec![BibleRenderGraphInfluence {
            influence_id,
            timeline_node_id,
            source_layer: StoryLevel::Scene,
            influence_kind: ContextInfluenceKind::Direct,
            confidence: 0.9,
            reason: "Scene uses Ada at the beach.".to_string(),
            provenance: ContextInfluenceProvenance::AiSelected,
            bible_node_id: Some(ada_id),
            bible_edge_id: Some(edge_id),
            sort_order: 0,
        }],
    }
}
