use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use eidetic_bevy_timeline::{
    TimelineNativeWindowControlHandle, TimelineNativeWindowProjectionUpdateError,
    TimelineNativeWindowRunnerConfig, TimelineRendererCommand,
    run_controlled_minimal_timeline_native_window,
};
use eidetic_core::contracts::TimelineRenderProjection;

#[derive(Debug)]
pub struct TimelineRendererWindowThreadHandle {
    control_handle: TimelineNativeWindowControlHandle,
    completion_receiver: mpsc::Receiver<TimelineRendererWindowThreadResult>,
    join_handle: Option<JoinHandle<()>>,
    result: Option<TimelineRendererWindowThreadResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineRendererWindowThreadStatus {
    pub running: bool,
    pub ready: bool,
    pub visible: bool,
    pub close_requested: bool,
    pub result: Option<TimelineRendererWindowThreadResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineRendererWindowThreadResult {
    Completed,
    Panicked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineRendererWindowProjectionUpdateError {
    QueueFull,
    WindowClosed,
}

impl TimelineRendererWindowThreadHandle {
    pub fn start(config: TimelineNativeWindowRunnerConfig) -> std::io::Result<Self> {
        Self::start_with(config, run_controlled_minimal_timeline_native_window)
    }

    pub fn start_with(
        config: TimelineNativeWindowRunnerConfig,
        runner: impl FnOnce(TimelineNativeWindowRunnerConfig, TimelineNativeWindowControlHandle)
        + Send
        + 'static,
    ) -> std::io::Result<Self> {
        let control_handle = TimelineNativeWindowControlHandle::new();
        let thread_control_handle = control_handle.clone();
        let (completion_sender, completion_receiver) = mpsc::channel();
        let join_handle = thread::Builder::new()
            .name("eidetic-timeline-renderer-window".to_string())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    runner(config, thread_control_handle);
                }))
                .map(|_| TimelineRendererWindowThreadResult::Completed)
                .unwrap_or(TimelineRendererWindowThreadResult::Panicked);
                let _ = completion_sender.send(result);
            })?;

        Ok(Self {
            control_handle,
            completion_receiver,
            join_handle: Some(join_handle),
            result: None,
        })
    }

    pub fn request_close(&self) {
        self.control_handle.request_close();
    }

    pub fn request_show(&self) {
        self.control_handle.request_show();
    }

    pub fn request_hide(&self) {
        self.control_handle.request_hide();
    }

    pub fn update_projection(
        &self,
        projection: TimelineRenderProjection,
    ) -> Result<(), TimelineRendererWindowProjectionUpdateError> {
        self.control_handle
            .request_projection_update(projection)
            .map_err(TimelineRendererWindowProjectionUpdateError::from)
    }

    pub fn drain_commands(&self) -> Vec<TimelineRendererCommand> {
        self.control_handle.drain_commands()
    }

    pub fn status(&mut self) -> TimelineRendererWindowThreadStatus {
        self.refresh_result();
        TimelineRendererWindowThreadStatus {
            running: self.result.is_none(),
            ready: self.control_handle.ready(),
            visible: self.control_handle.visible(),
            close_requested: self.control_handle.close_requested(),
            result: self.result,
        }
    }

    pub fn stop(&mut self, timeout: Duration) -> TimelineRendererWindowThreadStatus {
        self.request_close();
        self.refresh_result();
        if self.result.is_none()
            && let Ok(result) = self.completion_receiver.recv_timeout(timeout)
        {
            self.result = Some(result);
        }
        if self.result.is_some()
            && let Some(join_handle) = self.join_handle.take()
        {
            let _ = join_handle.join();
        }
        self.status()
    }

    pub fn join_completed(&mut self) -> TimelineRendererWindowThreadStatus {
        self.refresh_result();
        if self.result.is_some()
            && let Some(join_handle) = self.join_handle.take()
        {
            let _ = join_handle.join();
        }
        self.status()
    }

    fn refresh_result(&mut self) {
        if self.result.is_some() {
            return;
        }
        if let Ok(result) = self.completion_receiver.try_recv() {
            self.result = Some(result);
        }
    }
}

impl From<TimelineNativeWindowProjectionUpdateError>
    for TimelineRendererWindowProjectionUpdateError
{
    fn from(error: TimelineNativeWindowProjectionUpdateError) -> Self {
        match error {
            TimelineNativeWindowProjectionUpdateError::QueueFull => Self::QueueFull,
            TimelineNativeWindowProjectionUpdateError::WindowClosed => Self::WindowClosed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_renderer_window_thread_reports_completion() {
        let mut handle = TimelineRendererWindowThreadHandle::start_with(
            TimelineNativeWindowRunnerConfig::minimal_smoke(true),
            |_config, control| {
                control.mark_ready();
                control.mark_visible(true);
            },
        )
        .unwrap();

        let status = handle.stop(Duration::from_millis(200));

        assert!(!status.running);
        assert!(status.ready);
        assert!(status.visible);
        assert!(status.close_requested);
        assert_eq!(
            status.result,
            Some(TimelineRendererWindowThreadResult::Completed)
        );
    }

    #[test]
    fn timeline_renderer_window_thread_can_request_bounded_close() {
        let mut handle = TimelineRendererWindowThreadHandle::start_with(
            TimelineNativeWindowRunnerConfig::minimal_smoke(true),
            |_config, control| {
                while !control.close_requested() {
                    std::thread::sleep(Duration::from_millis(1));
                }
            },
        )
        .unwrap();

        let status = handle.stop(Duration::from_millis(200));

        assert!(!status.running);
        assert!(status.close_requested);
        assert_eq!(
            status.result,
            Some(TimelineRendererWindowThreadResult::Completed)
        );
    }

    #[test]
    fn timeline_renderer_window_thread_reports_panic() {
        let mut handle = TimelineRendererWindowThreadHandle::start_with(
            TimelineNativeWindowRunnerConfig::minimal_smoke(true),
            |_config, _control| {
                panic!("timeline native window panic");
            },
        )
        .unwrap();

        let status = handle.stop(Duration::from_millis(200));

        assert!(!status.running);
        assert_eq!(
            status.result,
            Some(TimelineRendererWindowThreadResult::Panicked)
        );
    }
}
