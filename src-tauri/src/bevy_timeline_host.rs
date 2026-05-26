use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use eidetic_bevy_timeline::{TimelineRendererApp, TimelineRendererCommand, TimelineRendererError};
use eidetic_core::contracts::TimelineRenderProjection;

use crate::renderer_window::DesktopRendererWindowKind;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelineHostStatus {
    pub renderer_window_kind: DesktopRendererWindowKind,
    pub running: bool,
    pub renderer_scene_ready: bool,
    pub track_count: usize,
    pub clip_count: usize,
    pub relationship_count: usize,
    pub affect_overlay_count: usize,
    pub queued_command_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineHostError {
    Renderer(String),
    RendererPanic,
    QueueFull,
    OwnerReplyTimeout,
    OwnerStopped,
}

type TimelineHostResult<T> = Result<T, TimelineHostError>;

pub const TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY: usize = 128;
pub const TIMELINE_RENDERER_REPLY_TIMEOUT_MS: u64 = 2_000;

pub struct DesktopTimelineHost {
    renderer: Option<TimelineRendererApp>,
    last_error: Option<String>,
}

impl Default for DesktopTimelineHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopTimelineHost {
    pub fn new() -> Self {
        Self {
            renderer: None,
            last_error: None,
        }
    }

    pub fn renderer_unavailable_status(message: String) -> TimelineHostStatus {
        TimelineHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::Timeline,
            running: false,
            renderer_scene_ready: false,
            track_count: 0,
            clip_count: 0,
            relationship_count: 0,
            affect_overlay_count: 0,
            queued_command_count: 0,
            last_error: Some(message),
        }
    }

    pub fn start(&mut self) -> TimelineHostStatus {
        if self.renderer.is_none() {
            self.renderer = Some(TimelineRendererApp::new());
        }
        self.last_error = None;
        self.status()
    }

    pub fn stop(&mut self) -> TimelineHostStatus {
        self.renderer = None;
        self.last_error = None;
        self.status()
    }

    pub fn set_projection(
        &mut self,
        projection: TimelineRenderProjection,
    ) -> Result<TimelineHostStatus, TimelineHostError> {
        self.start();
        let Some(renderer) = self.renderer.as_mut() else {
            return Err(TimelineHostError::Renderer(
                TimelineRendererError::MissingProjection.to_string(),
            ));
        };

        Self::catch_renderer_panic(|| renderer.set_projection(projection)).map_err(|error| {
            self.last_error = Some(error_label(&error));
            error
        })?;
        self.last_error = None;
        Ok(self.status())
    }

    pub fn drain_commands(&mut self) -> Vec<TimelineRendererCommand> {
        self.renderer
            .as_mut()
            .map(TimelineRendererApp::drain_commands)
            .unwrap_or_default()
    }

    pub fn status(&self) -> TimelineHostStatus {
        let (track_count, clip_count) = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_counts)
            .unwrap_or_default();
        let relationship_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_relationship_count)
            .unwrap_or_default();
        let affect_overlay_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_affect_overlay_count)
            .unwrap_or_default();
        let queued_command_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::queued_command_count)
            .unwrap_or_default();

        TimelineHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::Timeline,
            running: self.renderer.is_some(),
            renderer_scene_ready: self.renderer.is_some(),
            track_count,
            clip_count,
            relationship_count,
            affect_overlay_count,
            queued_command_count,
            last_error: self.last_error.clone(),
        }
    }

    fn catch_renderer_panic<T>(operation: impl FnOnce() -> T) -> Result<T, TimelineHostError> {
        catch_unwind(AssertUnwindSafe(operation)).map_err(|_| TimelineHostError::RendererPanic)
    }
}

enum TimelineHostRequest {
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
            TimelineHostRequest::SetProjection { projection, reply } => {
                let _ = reply.send(host.set_projection(projection));
            }
            TimelineHostRequest::DrainCommands { reply } => {
                let _ = reply.send(Ok(host.drain_commands()));
            }
            TimelineHostRequest::Status { reply } => {
                let _ = reply.send(Ok(host.status()));
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

fn error_label(error: &TimelineHostError) -> String {
    match error {
        TimelineHostError::Renderer(message) => message.clone(),
        TimelineHostError::RendererPanic => "timeline renderer panicked".to_string(),
        TimelineHostError::QueueFull => "timeline renderer command queue is full".to_string(),
        TimelineHostError::OwnerReplyTimeout => {
            "timeline renderer owner reply timed out".to_string()
        }
        TimelineHostError::OwnerStopped => "timeline renderer owner stopped".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::{
        TimelineRenderClip, TimelineRenderProjection, TimelineRenderTrack,
    };
    use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
    use eidetic_core::timeline::track::TrackId;

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
        assert!(status.renderer_scene_ready);
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
        assert_eq!(status.track_count, 0);
        assert_eq!(status.clip_count, 0);
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

    fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
        let track_id = TrackId::new();
        TimelineRenderProjection {
            total_duration_ms: 10_000,
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
}
