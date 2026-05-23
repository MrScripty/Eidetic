use std::sync::{Mutex, mpsc};
use std::thread::{self, JoinHandle};

use eidetic_bevy_bible_graph::{
    BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, BibleGraphRendererCommand,
    BibleGraphVisualSnapshot,
};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};

use super::{
    BibleGraphHostError, BibleGraphHostResult, BibleGraphHostStatus, DesktopBibleGraphHost,
};

pub const GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY: usize =
    BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY;

enum BibleGraphHostRequest {
    Start {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
    SetProjection {
        projection: BibleRenderGraphProjection,
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
    SetRendererWindowBounds {
        width_px: u32,
        height_px: u32,
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
    SelectNode {
        node_id: BibleGraphNodeId,
        reply: mpsc::Sender<BibleGraphHostResult<()>>,
    },
    InspectNode {
        node_id: BibleGraphNodeId,
        reply: mpsc::Sender<BibleGraphHostResult<()>>,
    },
    SelectEdge {
        edge_id: BibleGraphEdgeId,
        reply: mpsc::Sender<BibleGraphHostResult<()>>,
    },
    SelectInfluence {
        influence_id: ContextInfluenceId,
        reply: mpsc::Sender<BibleGraphHostResult<()>>,
    },
    DrainCommands {
        reply: mpsc::Sender<BibleGraphHostResult<Vec<BibleGraphRendererCommand>>>,
    },
    VisualSnapshot {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphVisualSnapshot>>,
    },
    Status {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
    Stop {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
}

pub struct DesktopBibleGraphRendererOwner {
    sender: mpsc::SyncSender<BibleGraphHostRequest>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
}

impl DesktopBibleGraphRendererOwner {
    pub fn start() -> BibleGraphHostResult<Self> {
        let (sender, receiver) = mpsc::sync_channel(GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY);
        let join_handle = thread::Builder::new()
            .name("eidetic-bevy-bible-graph".to_string())
            .spawn(move || run_renderer_owner(receiver))
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;

        Ok(Self {
            sender,
            join_handle: Mutex::new(Some(join_handle)),
        })
    }

    pub fn start_renderer(&self) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::Start { reply })?;
        receive_reply(receiver)
    }

    pub fn set_projection(
        &self,
        projection: BibleRenderGraphProjection,
    ) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::SetProjection { projection, reply })?;
        receive_reply(receiver)
    }

    pub fn set_renderer_window_bounds(
        &self,
        width_px: u32,
        height_px: u32,
    ) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::SetRendererWindowBounds {
            width_px,
            height_px,
            reply,
        })?;
        receive_reply(receiver)
    }

    pub fn select_node(&self, node_id: BibleGraphNodeId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::SelectNode { node_id, reply })?;
        receive_reply(receiver)
    }

    pub fn inspect_node(&self, node_id: BibleGraphNodeId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::InspectNode { node_id, reply })?;
        receive_reply(receiver)
    }

    pub fn select_edge(&self, edge_id: BibleGraphEdgeId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::SelectEdge { edge_id, reply })?;
        receive_reply(receiver)
    }

    pub fn select_influence(&self, influence_id: ContextInfluenceId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::SelectInfluence {
            influence_id,
            reply,
        })?;
        receive_reply(receiver)
    }

    pub fn drain_commands(&self) -> BibleGraphHostResult<Vec<BibleGraphRendererCommand>> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::DrainCommands { reply })?;
        receive_reply(receiver)
    }

    pub fn visual_snapshot(&self) -> BibleGraphHostResult<BibleGraphVisualSnapshot> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::VisualSnapshot { reply })?;
        receive_reply(receiver)
    }

    pub fn status(&self) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::Status { reply })?;
        receive_reply(receiver)
    }

    pub fn stop(&self) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.enqueue(BibleGraphHostRequest::Stop { reply })?;
        let status = receive_reply(receiver)?;
        if let Ok(mut join_handle) = self.join_handle.lock()
            && let Some(join_handle) = join_handle.take()
        {
            join_handle
                .join()
                .map_err(|_| BibleGraphHostError::RendererPanic)?;
        }
        Ok(status)
    }

    fn enqueue(&self, request: BibleGraphHostRequest) -> BibleGraphHostResult<()> {
        match self.sender.try_send(request) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_)) => Err(BibleGraphHostError::QueueFull),
            Err(mpsc::TrySendError::Disconnected(_)) => Err(BibleGraphHostError::OwnerStopped),
        }
    }
}

impl Drop for DesktopBibleGraphRendererOwner {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn receive_reply<T>(receiver: mpsc::Receiver<BibleGraphHostResult<T>>) -> BibleGraphHostResult<T> {
    receiver
        .recv()
        .map_err(|_| BibleGraphHostError::OwnerStopped)?
}

fn run_renderer_owner(receiver: mpsc::Receiver<BibleGraphHostRequest>) {
    let mut host = DesktopBibleGraphHost::new();

    for request in receiver {
        match request {
            BibleGraphHostRequest::Start { reply } => {
                let _ = reply.send(host.start());
            }
            BibleGraphHostRequest::SetProjection { projection, reply } => {
                let _ = reply.send(host.set_projection(projection));
            }
            BibleGraphHostRequest::SetRendererWindowBounds {
                width_px,
                height_px,
                reply,
            } => {
                let _ = reply.send(host.set_renderer_window_bounds(width_px, height_px));
            }
            BibleGraphHostRequest::SelectNode { node_id, reply } => {
                let _ = reply.send(host.select_node(node_id));
            }
            BibleGraphHostRequest::InspectNode { node_id, reply } => {
                let _ = reply.send(host.inspect_node(node_id));
            }
            BibleGraphHostRequest::SelectEdge { edge_id, reply } => {
                let _ = reply.send(host.select_edge(edge_id));
            }
            BibleGraphHostRequest::SelectInfluence {
                influence_id,
                reply,
            } => {
                let _ = reply.send(host.select_influence(influence_id));
            }
            BibleGraphHostRequest::DrainCommands { reply } => {
                let _ = reply.send(Ok(host.drain_commands()));
            }
            BibleGraphHostRequest::VisualSnapshot { reply } => {
                let _ = reply.send(host.visual_snapshot());
            }
            BibleGraphHostRequest::Status { reply } => {
                let _ = reply.send(Ok(host.status()));
            }
            BibleGraphHostRequest::Stop { reply } => {
                let _ = reply.send(Ok(host.stop()));
                break;
            }
        }
    }
}
