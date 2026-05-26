use std::sync::{Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use eidetic_bevy_timeline::TimelineRendererCommand;
use eidetic_core::contracts::TimelineRenderProjection;

use crate::bevy_timeline_host::{
    DesktopTimelineHost, TimelineHostError, TimelineHostResult, TimelineHostStatus,
};

pub const TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY: usize = 128;
pub const TIMELINE_RENDERER_REPLY_TIMEOUT_MS: u64 = 2_000;

enum TimelineHostRequest {
    OpenRenderer {
        projection: TimelineRenderProjection,
        reply: mpsc::Sender<TimelineHostResult<TimelineHostStatus>>,
    },
    SetProjection {
        projection: TimelineRenderProjection,
        reply: mpsc::Sender<TimelineHostResult<TimelineHostStatus>>,
    },
    DrainCommands {
        reply: mpsc::Sender<TimelineHostResult<Vec<TimelineRendererCommand>>>,
    },
    Status {
        reply: mpsc::Sender<TimelineHostResult<TimelineHostStatus>>,
    },
    FocusRenderer {
        reply: mpsc::Sender<TimelineHostResult<TimelineHostStatus>>,
    },
    CloseRenderer {
        reply: mpsc::Sender<TimelineHostResult<TimelineHostStatus>>,
    },
    Stop {
        reply: mpsc::Sender<TimelineHostResult<TimelineHostStatus>>,
    },
}

pub struct DesktopTimelineRendererOwner {
    sender: Option<mpsc::SyncSender<TimelineHostRequest>>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
    unavailable_status: Option<TimelineHostStatus>,
}

impl DesktopTimelineRendererOwner {
    pub fn start() -> TimelineHostResult<Self> {
        let (sender, receiver) = mpsc::sync_channel(TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY);
        let join_handle = thread::Builder::new()
            .name("eidetic-bevy-timeline".to_string())
            .spawn(move || run_timeline_owner(DesktopTimelineHost::new(), receiver))
            .map_err(|_| TimelineHostError::OwnerStopped)?;

        Ok(Self {
            sender: Some(sender),
            join_handle: Mutex::new(Some(join_handle)),
            unavailable_status: None,
        })
    }

    pub fn unavailable(message: String) -> Self {
        Self {
            sender: None,
            join_handle: Mutex::new(None),
            unavailable_status: Some(DesktopTimelineHost::renderer_unavailable_status(message)),
        }
    }

    pub fn set_projection(
        &self,
        projection: TimelineRenderProjection,
    ) -> TimelineHostResult<TimelineHostStatus> {
        if let Some(status) = self.unavailable_status() {
            return Ok(status);
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::SetProjection { projection, reply })?;
        receive_reply(receiver)
    }

    pub fn open_renderer(
        &self,
        projection: TimelineRenderProjection,
    ) -> TimelineHostResult<TimelineHostStatus> {
        if let Some(status) = self.unavailable_status() {
            return Ok(status);
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::OpenRenderer { projection, reply })?;
        receive_reply(receiver)
    }

    pub fn drain_commands(&self) -> TimelineHostResult<Vec<TimelineRendererCommand>> {
        if self.unavailable_status().is_some() {
            return Ok(Vec::new());
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::DrainCommands { reply })?;
        receive_reply(receiver)
    }

    pub fn status(&self) -> TimelineHostResult<TimelineHostStatus> {
        if let Some(status) = self.unavailable_status() {
            return Ok(status);
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::Status { reply })?;
        receive_reply(receiver)
    }

    pub fn close_renderer(&self) -> TimelineHostResult<TimelineHostStatus> {
        if let Some(status) = self.unavailable_status() {
            return Ok(status);
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::CloseRenderer { reply })?;
        receive_reply(receiver)
    }

    pub fn focus_renderer(&self) -> TimelineHostResult<TimelineHostStatus> {
        if let Some(status) = self.unavailable_status() {
            return Ok(status);
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::FocusRenderer { reply })?;
        receive_reply(receiver)
    }

    pub fn stop(&self) -> TimelineHostResult<TimelineHostStatus> {
        if let Some(status) = self.unavailable_status() {
            return Ok(status);
        }
        let (reply, receiver) = mpsc::channel();
        self.enqueue(TimelineHostRequest::Stop { reply })?;
        let status = receive_reply(receiver)?;
        if let Ok(mut join_handle) = self.join_handle.lock()
            && let Some(join_handle) = join_handle.take()
        {
            join_handle
                .join()
                .map_err(|_| TimelineHostError::RendererPanic)?;
        }
        Ok(status)
    }

    fn enqueue(&self, request: TimelineHostRequest) -> TimelineHostResult<()> {
        let Some(sender) = &self.sender else {
            return Err(TimelineHostError::OwnerStopped);
        };
        match sender.try_send(request) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_)) => Err(TimelineHostError::QueueFull),
            Err(mpsc::TrySendError::Disconnected(_)) => Err(TimelineHostError::OwnerStopped),
        }
    }

    fn unavailable_status(&self) -> Option<TimelineHostStatus> {
        self.unavailable_status.clone()
    }
}

impl Drop for DesktopTimelineRendererOwner {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn receive_reply<T>(receiver: mpsc::Receiver<TimelineHostResult<T>>) -> TimelineHostResult<T> {
    match receiver.recv_timeout(Duration::from_millis(TIMELINE_RENDERER_REPLY_TIMEOUT_MS)) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(TimelineHostError::OwnerReplyTimeout),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(TimelineHostError::OwnerStopped),
    }
}

fn run_timeline_owner(
    mut host: DesktopTimelineHost,
    receiver: mpsc::Receiver<TimelineHostRequest>,
) {
    for request in receiver {
        match request {
            TimelineHostRequest::OpenRenderer { projection, reply } => {
                let _ = reply.send(host.open_renderer(projection));
            }
            TimelineHostRequest::SetProjection { projection, reply } => {
                let _ = reply.send(host.set_projection(projection));
            }
            TimelineHostRequest::DrainCommands { reply } => {
                let _ = reply.send(Ok(host.drain_commands()));
            }
            TimelineHostRequest::Status { reply } => {
                let _ = reply.send(Ok(host.status()));
            }
            TimelineHostRequest::FocusRenderer { reply } => {
                let _ = reply.send(Ok(host.focus()));
            }
            TimelineHostRequest::CloseRenderer { reply } => {
                let _ = reply.send(Ok(host.stop()));
            }
            TimelineHostRequest::Stop { reply } => {
                let _ = reply.send(Ok(host.stop()));
                break;
            }
        }
    }
}
