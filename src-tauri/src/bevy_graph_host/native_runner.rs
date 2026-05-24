use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    BibleGraphRendererWindowPlatform, BibleGraphRendererWindowStrategy,
    BibleGraphRendererWindowStrategyStatus, NativeRendererPlatformStrategy,
};

pub const NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY: usize = 16;
pub const NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRendererRunnerStatus {
    pub strategy: BibleGraphRendererWindowStrategy,
    pub platform: BibleGraphRendererWindowPlatform,
    pub lifecycle: NativeRendererRunnerLifecycle,
    pub capability: BibleGraphRendererWindowCapability,
    pub capability_reason: BibleGraphRendererWindowCapabilityReason,
    pub visible_window_supported: bool,
    pub window_visible: bool,
    pub window_ready: bool,
    pub focus_supported: bool,
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
        let (sender, receiver) = mpsc::sync_channel(NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY);
        let join_handle = thread::Builder::new()
            .name("eidetic-native-renderer-runner".to_string())
            .spawn(move || run_native_renderer_runner(strategy, receiver))?;

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

    fn status(&self) -> NativeRendererRunnerStatus {
        self.request(|reply| NativeRendererRunnerRequest::Status { reply })
    }
}

impl Drop for NativeRendererRunnerHandle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Debug, Default)]
pub struct PendingNativeRendererRunner {
    strategy: Option<NativeRendererPlatformStrategy>,
    open_requested: bool,
}

impl PendingNativeRendererRunner {
    pub fn for_strategy(strategy: NativeRendererPlatformStrategy) -> Self {
        Self {
            strategy: Some(strategy),
            open_requested: false,
        }
    }
}

impl NativeRendererRunner for PendingNativeRendererRunner {
    fn open(&mut self) -> NativeRendererRunnerStatus {
        self.open_requested = true;
        self.status()
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        self.open_requested = false;
        self.status()
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        self.status()
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        let strategy = self
            .strategy
            .map(NativeRendererPlatformStrategy::status)
            .unwrap_or_else(BibleGraphRendererWindowStrategyStatus::current);
        NativeRendererRunnerStatus {
            strategy: strategy.strategy,
            platform: strategy.platform,
            lifecycle: if self.open_requested {
                NativeRendererRunnerLifecycle::OpenRequested
            } else {
                NativeRendererRunnerLifecycle::Closed
            },
            capability: strategy.capability,
            capability_reason: strategy.capability_reason,
            visible_window_supported: strategy.visible_window_supported,
            window_visible: false,
            window_ready: false,
            focus_supported: false,
            last_error: None,
        }
    }
}

#[cfg(test)]
impl PendingNativeRendererRunner {
    pub fn open_requested(&self) -> bool {
        self.open_requested
    }
}

fn run_native_renderer_runner(
    strategy: NativeRendererPlatformStrategy,
    receiver: mpsc::Receiver<NativeRendererRunnerRequest>,
) {
    let mut runner = PendingNativeRendererRunner::for_strategy(strategy);

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
            NativeRendererRunnerRequest::Status { reply } => {
                let _ = reply.send(runner.status());
            }
            NativeRendererRunnerRequest::Stop { reply } => {
                let _ = reply.send(runner.close());
                break;
            }
        }
    }
}

fn pending_runner_unavailable_status(message: String) -> NativeRendererRunnerStatus {
    NativeRendererRunnerStatus {
        capability_reason: BibleGraphRendererWindowCapabilityReason::RunnerError,
        last_error: Some(message),
        ..PendingNativeRendererRunner::default().status()
    }
}
