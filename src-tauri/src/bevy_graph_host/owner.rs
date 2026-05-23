use std::sync::{Mutex, mpsc};
use std::thread::{self, JoinHandle};

use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};

use super::{
    BibleGraphHostError, BibleGraphHostResult, BibleGraphHostStatus, DesktopBibleGraphHost,
};

enum BibleGraphHostRequest {
    Start {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
    SetProjection {
        projection: BibleRenderGraphProjection,
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
    Status {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
    Stop {
        reply: mpsc::Sender<BibleGraphHostResult<BibleGraphHostStatus>>,
    },
}

pub struct DesktopBibleGraphRendererOwner {
    sender: mpsc::Sender<BibleGraphHostRequest>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
}

impl DesktopBibleGraphRendererOwner {
    pub fn start() -> BibleGraphHostResult<Self> {
        let (sender, receiver) = mpsc::channel();
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
        self.sender
            .send(BibleGraphHostRequest::Start { reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn set_projection(
        &self,
        projection: BibleRenderGraphProjection,
    ) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::SetProjection { projection, reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn select_node(&self, node_id: BibleGraphNodeId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::SelectNode { node_id, reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn inspect_node(&self, node_id: BibleGraphNodeId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::InspectNode { node_id, reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn select_edge(&self, edge_id: BibleGraphEdgeId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::SelectEdge { edge_id, reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn select_influence(&self, influence_id: ContextInfluenceId) -> BibleGraphHostResult<()> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::SelectInfluence {
                influence_id,
                reply,
            })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn drain_commands(&self) -> BibleGraphHostResult<Vec<BibleGraphRendererCommand>> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::DrainCommands { reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn status(&self) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::Status { reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
        receive_reply(receiver)
    }

    pub fn stop(&self) -> BibleGraphHostResult<BibleGraphHostStatus> {
        let (reply, receiver) = mpsc::channel();
        self.sender
            .send(BibleGraphHostRequest::Stop { reply })
            .map_err(|_| BibleGraphHostError::OwnerStopped)?;
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
