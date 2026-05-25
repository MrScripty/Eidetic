use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use eidetic_bevy_bible_graph::{
    BibleGraphCameraCommand, BibleGraphNativeVisualStatus, BibleGraphNativeWindowControlHandle,
    BibleGraphNativeWindowRunnerConfig, BibleGraphRendererCommand,
    run_controlled_minimal_bible_graph_native_window,
};
use eidetic_core::contracts::BibleRenderGraphProjection;

#[derive(Debug)]
pub struct NativeRendererWindowThreadHandle {
    control_handle: BibleGraphNativeWindowControlHandle,
    completion_receiver: mpsc::Receiver<NativeRendererWindowThreadResult>,
    join_handle: Option<JoinHandle<()>>,
    result: Option<NativeRendererWindowThreadResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeRendererWindowThreadStatus {
    pub running: bool,
    pub ready: bool,
    pub visible: bool,
    pub native_visual_counts: BibleGraphNativeVisualStatus,
    pub close_requested: bool,
    pub result: Option<NativeRendererWindowThreadResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRendererWindowThreadResult {
    Completed,
    Panicked,
}

impl NativeRendererWindowThreadHandle {
    pub fn start(config: BibleGraphNativeWindowRunnerConfig) -> std::io::Result<Self> {
        Self::start_with(config, run_controlled_minimal_bible_graph_native_window)
    }

    pub fn start_with(
        config: BibleGraphNativeWindowRunnerConfig,
        runner: impl FnOnce(BibleGraphNativeWindowRunnerConfig, BibleGraphNativeWindowControlHandle)
        + Send
        + 'static,
    ) -> std::io::Result<Self> {
        let control_handle = BibleGraphNativeWindowControlHandle::new();
        let thread_control_handle = control_handle.clone();
        let (completion_sender, completion_receiver) = mpsc::channel();
        let join_handle = thread::Builder::new()
            .name("eidetic-native-renderer-window".to_string())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    runner(config, thread_control_handle);
                }))
                .map(|_| NativeRendererWindowThreadResult::Completed)
                .unwrap_or(NativeRendererWindowThreadResult::Panicked);
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

    pub fn set_projection(&self, projection: BibleRenderGraphProjection) {
        self.control_handle.set_projection(projection);
    }

    pub fn apply_camera_command(
        &self,
        command: BibleGraphCameraCommand,
    ) -> Result<(), eidetic_bevy_bible_graph::BibleGraphRendererError> {
        self.control_handle.push_camera_command(command)
    }

    pub fn drain_commands(&self) -> Vec<BibleGraphRendererCommand> {
        self.control_handle.drain_commands()
    }

    pub fn status(&mut self) -> NativeRendererWindowThreadStatus {
        self.refresh_result();
        NativeRendererWindowThreadStatus {
            running: self.result.is_none(),
            ready: self.control_handle.ready(),
            visible: self.control_handle.visible(),
            native_visual_counts: self.control_handle.native_visual_counts(),
            close_requested: self.control_handle.close_requested(),
            result: self.result,
        }
    }

    pub fn stop(&mut self, timeout: Duration) -> NativeRendererWindowThreadStatus {
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

    pub fn join_completed(&mut self) -> NativeRendererWindowThreadStatus {
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
