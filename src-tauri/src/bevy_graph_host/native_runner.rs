use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    BibleGraphRendererWindowPlatform, BibleGraphRendererWindowStrategy,
    NativeRendererPlatformStrategy, NativeRendererSupervisor, NativeRendererSupervisorLifecycle,
    NativeRendererThreadingModel, NativeRendererWindowThreadHandle,
};
use eidetic_bevy_bible_graph::{BibleGraphNativeWindowRunnerConfig, BibleGraphRendererCommand};
use eidetic_core::contracts::BibleRenderGraphProjection;

pub const NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY: usize = 16;
pub const NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRendererRunnerStatus {
    pub strategy: BibleGraphRendererWindowStrategy,
    pub platform: BibleGraphRendererWindowPlatform,
    pub lifecycle: NativeRendererRunnerLifecycle,
    pub supervisor_lifecycle: NativeRendererSupervisorLifecycle,
    pub threading_model: NativeRendererThreadingModel,
    pub capability: BibleGraphRendererWindowCapability,
    pub capability_reason: BibleGraphRendererWindowCapabilityReason,
    pub verified_support: bool,
    pub visible_window_supported: bool,
    pub window_visible: bool,
    pub window_ready: bool,
    pub focus_supported: bool,
    pub native_visual_node_count: usize,
    pub native_visual_edge_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeRendererRunnerLifecycle {
    Closed,
    OpenRequested,
    Visible,
}

pub trait NativeRendererRunner {
    fn open(&mut self) -> NativeRendererRunnerStatus;
    fn close(&mut self) -> NativeRendererRunnerStatus;
    fn focus(&mut self) -> NativeRendererRunnerStatus;
    fn set_projection(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> NativeRendererRunnerStatus;
    fn drain_commands(&mut self) -> Vec<BibleGraphRendererCommand>;
    fn status(&self) -> NativeRendererRunnerStatus;
}

enum NativeRendererRunnerRequest {
    Open {
        reply: mpsc::Sender<NativeRendererRunnerStatus>,
    },
    Close {
        reply: mpsc::Sender<NativeRendererRunnerStatus>,
    },
    Focus {
        reply: mpsc::Sender<NativeRendererRunnerStatus>,
    },
    SetProjection {
        projection: BibleRenderGraphProjection,
        reply: mpsc::Sender<NativeRendererRunnerStatus>,
    },
    DrainCommands {
        reply: mpsc::Sender<Vec<BibleGraphRendererCommand>>,
    },
    Status {
        reply: mpsc::Sender<NativeRendererRunnerStatus>,
    },
    Stop {
        reply: mpsc::Sender<NativeRendererRunnerStatus>,
    },
}

pub struct NativeRendererRunnerHandle {
    sender: mpsc::SyncSender<NativeRendererRunnerRequest>,
    join_handle: Option<JoinHandle<()>>,
}

impl NativeRendererRunnerHandle {
    pub fn start_for_current_platform() -> std::io::Result<Self> {
        Self::start_for_strategy(NativeRendererPlatformStrategy::current())
    }

    pub fn start_for_strategy(strategy: NativeRendererPlatformStrategy) -> std::io::Result<Self> {
        Self::start_for_strategy_with_window_thread_start(
            strategy,
            NativeRendererWindowThreadHandle::start,
        )
    }

    pub(crate) fn start_for_strategy_with_window_thread_start(
        strategy: NativeRendererPlatformStrategy,
        window_thread_start: fn(
            BibleGraphNativeWindowRunnerConfig,
        ) -> std::io::Result<NativeRendererWindowThreadHandle>,
    ) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY);
        let join_handle = thread::Builder::new()
            .name("eidetic-native-renderer-runner".to_string())
            .spawn(move || {
                run_native_renderer_runner(strategy, window_thread_start, receiver);
            })?;

        Ok(Self {
            sender,
            join_handle: Some(join_handle),
        })
    }

    pub fn start_pending() -> std::io::Result<Self> {
        Self::start_for_strategy(NativeRendererPlatformStrategy::current())
    }

    fn request(
        &self,
        build: impl FnOnce(mpsc::Sender<NativeRendererRunnerStatus>) -> NativeRendererRunnerRequest,
    ) -> NativeRendererRunnerStatus {
        let (reply, receiver) = mpsc::channel();
        if let Err(error) = self.sender.try_send(build(reply)) {
            return pending_runner_unavailable_status(format!(
                "native renderer runner command queue unavailable: {error}"
            ));
        }
        receiver
            .recv_timeout(Duration::from_millis(
                NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS,
            ))
            .unwrap_or_else(|error| {
                pending_runner_unavailable_status(format!(
                    "native renderer runner reply unavailable: {error}"
                ))
            })
    }

    pub fn stop(&mut self) -> NativeRendererRunnerStatus {
        let (reply, receiver) = mpsc::channel();
        if let Err(error) = self
            .sender
            .try_send(NativeRendererRunnerRequest::Stop { reply })
        {
            return pending_runner_unavailable_status(format!(
                "native renderer runner stop unavailable: {error}"
            ));
        }

        let status = receiver
            .recv_timeout(Duration::from_millis(
                NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS,
            ))
            .unwrap_or_else(|error| {
                pending_runner_unavailable_status(format!(
                    "native renderer runner stop reply unavailable: {error}"
                ))
            });
        if let Some(join_handle) = self.join_handle.take()
            && join_handle.join().is_err()
        {
            return pending_runner_unavailable_status(
                "native renderer runner thread panicked during stop".to_string(),
            );
        }
        status
    }
}

impl NativeRendererRunner for NativeRendererRunnerHandle {
    fn open(&mut self) -> NativeRendererRunnerStatus {
        self.request(|reply| NativeRendererRunnerRequest::Open { reply })
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        self.request(|reply| NativeRendererRunnerRequest::Close { reply })
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        self.request(|reply| NativeRendererRunnerRequest::Focus { reply })
    }

    fn set_projection(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> NativeRendererRunnerStatus {
        self.request(|reply| NativeRendererRunnerRequest::SetProjection { projection, reply })
    }

    fn drain_commands(&mut self) -> Vec<BibleGraphRendererCommand> {
        let (reply, receiver) = mpsc::channel();
        if self
            .sender
            .try_send(NativeRendererRunnerRequest::DrainCommands { reply })
            .is_err()
        {
            return Vec::new();
        }
        receiver
            .recv_timeout(Duration::from_millis(
                NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS,
            ))
            .unwrap_or_default()
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        self.request(|reply| NativeRendererRunnerRequest::Status { reply })
    }
}

impl Drop for NativeRendererRunnerHandle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn run_native_renderer_runner(
    strategy: NativeRendererPlatformStrategy,
    window_thread_start: fn(
        BibleGraphNativeWindowRunnerConfig,
    ) -> std::io::Result<NativeRendererWindowThreadHandle>,
    receiver: mpsc::Receiver<NativeRendererRunnerRequest>,
) {
    let mut runner = NativeRendererSupervisor::for_strategy_with_window_thread_start(
        strategy,
        window_thread_start,
    );

    for request in receiver {
        match request {
            NativeRendererRunnerRequest::Open { reply } => {
                let _ = reply.send(runner.open());
            }
            NativeRendererRunnerRequest::Close { reply } => {
                let _ = reply.send(runner.close());
            }
            NativeRendererRunnerRequest::Focus { reply } => {
                let _ = reply.send(runner.focus());
            }
            NativeRendererRunnerRequest::SetProjection { projection, reply } => {
                let _ = reply.send(runner.set_projection(projection));
            }
            NativeRendererRunnerRequest::DrainCommands { reply } => {
                let _ = reply.send(runner.drain_commands());
            }
            NativeRendererRunnerRequest::Status { reply } => {
                let _ = reply.send(runner.refresh_status());
            }
            NativeRendererRunnerRequest::Stop { reply } => {
                let _ = reply.send(runner.shutdown());
                break;
            }
        }
    }
}

fn pending_runner_unavailable_status(message: String) -> NativeRendererRunnerStatus {
    NativeRendererSupervisor::failed_current_platform_status(message)
}
