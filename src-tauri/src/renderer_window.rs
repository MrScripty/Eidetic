#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowKind {
    BibleGraph,
    Timeline,
}

// Winit permits one event loop per process, even after the first loop exits.
// A hidden renderer keeps its original runner; a new runner must not recreate it.
static NATIVE_EVENT_LOOP: std::sync::OnceLock<DesktopRendererWindowKind> =
    std::sync::OnceLock::new();

pub(crate) fn reserve_native_event_loop(kind: DesktopRendererWindowKind) -> std::io::Result<()> {
    reserve_event_loop(&NATIVE_EVENT_LOOP, kind)
}

fn reserve_event_loop(
    reservation: &std::sync::OnceLock<DesktopRendererWindowKind>,
    kind: DesktopRendererWindowKind,
) -> std::io::Result<()> {
    reservation.set(kind).map_err(|_| {
        let owner = reservation.get().expect("event loop reservation is set");
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "The native {owner:?} renderer already reserved this process's winit event loop. \
                 Restart Eidetic and open the native {kind:?} renderer first. \
                 The main timeline remains available."
            ),
        )
    })
}

#[cfg(test)]
mod event_loop_tests {
    use super::*;
    use std::sync::OnceLock;

    #[test]
    fn rejects_a_second_renderer_and_recreation_with_actionable_error() {
        for owner in [
            DesktopRendererWindowKind::BibleGraph,
            DesktopRendererWindowKind::Timeline,
        ] {
            let reservation = OnceLock::new();
            reserve_event_loop(&reservation, owner).unwrap();
            for next in [
                DesktopRendererWindowKind::BibleGraph,
                DesktopRendererWindowKind::Timeline,
            ] {
                let error = reserve_event_loop(&reservation, next).unwrap_err();
                assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
                assert!(error.to_string().contains("Restart Eidetic"));
                assert_eq!(reservation.get(), Some(&owner));
            }
        }
    }

    #[test]
    fn concurrent_starts_admit_only_one_native_event_loop() {
        let reservation = OnceLock::new();
        let accepted = std::thread::scope(|scope| {
            let graph = scope
                .spawn(|| reserve_event_loop(&reservation, DesktopRendererWindowKind::BibleGraph));
            let timeline = scope
                .spawn(|| reserve_event_loop(&reservation, DesktopRendererWindowKind::Timeline));
            usize::from(graph.join().unwrap().is_ok())
                + usize::from(timeline.join().unwrap().is_ok())
        });
        assert_eq!(accepted, 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowStrategy {
    BevyWinitFloatingWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DesktopRendererWindowStrategyStatus {
    pub strategy: DesktopRendererWindowStrategy,
    pub platform: DesktopRendererWindowPlatform,
    pub capability: DesktopRendererWindowCapability,
    pub capability_reason: DesktopRendererWindowCapabilityReason,
    pub verified_support: bool,
    pub visible_window_supported: bool,
}

impl DesktopRendererWindowStrategyStatus {
    pub fn pending_native_runner_current_platform() -> Self {
        Self::current_platform_with_capability(
            DesktopRendererWindowCapability::PendingNativeRunner,
            DesktopRendererWindowCapabilityReason::PendingNativeRunner,
        )
    }

    pub fn runner_error_current_platform() -> Self {
        Self::current_platform_with_capability(
            DesktopRendererWindowCapability::RunnerError,
            DesktopRendererWindowCapabilityReason::RunnerError,
        )
    }

    fn current_platform_with_capability(
        capability: DesktopRendererWindowCapability,
        capability_reason: DesktopRendererWindowCapabilityReason,
    ) -> Self {
        Self {
            strategy: DesktopRendererWindowStrategy::BevyWinitFloatingWindow,
            platform: current_desktop_renderer_window_platform(),
            capability,
            capability_reason,
            verified_support: capability.verified_support(),
            visible_window_supported: capability.visible_window_supported(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowPlatform {
    Linux,
    Macos,
    Windows,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowCapability {
    PendingNativeRunner,
    PlatformUnproven,
    PlatformUnsupported,
    RunnerError,
    VerifiedSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowCapabilityReason {
    PendingNativeRunner,
    PlatformUnproven,
    PlatformUnsupported,
    RunnerError,
    VerifiedSupport,
}

impl DesktopRendererWindowCapability {
    pub fn verified_support(self) -> bool {
        matches!(self, Self::VerifiedSupport)
    }

    pub fn visible_window_supported(self) -> bool {
        matches!(self, Self::VerifiedSupport)
    }
}

pub fn current_desktop_renderer_window_platform() -> DesktopRendererWindowPlatform {
    if cfg!(target_os = "linux") {
        DesktopRendererWindowPlatform::Linux
    } else if cfg!(target_os = "macos") {
        DesktopRendererWindowPlatform::Macos
    } else if cfg!(target_os = "windows") {
        DesktopRendererWindowPlatform::Windows
    } else {
        DesktopRendererWindowPlatform::Unsupported
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowLifecycle {
    Closed,
    SceneStarting,
    SceneReadyPendingNativeRunner,
    Visible,
}

impl DesktopRendererWindowLifecycle {
    pub fn from_state(running: bool, scene_ready: bool, window_visible: bool) -> Self {
        match (running, scene_ready, window_visible) {
            (true, true, true) => Self::Visible,
            (true, true, false) => Self::SceneReadyPendingNativeRunner,
            (true, false, _) => Self::SceneStarting,
            (false, _, _) => Self::Closed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererRunnerLifecycle {
    Closed,
    OpenRequested,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererSupervisorLifecycle {
    NotStarted,
    Starting,
    Running,
    Closing,
    Closed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererThreadingModel {
    WorkerThread,
    MainThread,
    Unsupported,
}
