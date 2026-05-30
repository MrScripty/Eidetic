use bevy::prelude::{App, Resource};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphNeighborhood, BibleRenderGraphProjection,
    ContextInfluenceId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod category;
mod scene;
mod visual;
mod visual_3d;
mod workspace;

#[cfg(feature = "native_render")]
mod native_render;
#[cfg(feature = "native_render")]
mod native_text_editor;

#[cfg(feature = "native_render")]
pub use native_render::{
    BibleGraphNativeCamera, BibleGraphNativeEdgeVisual, BibleGraphNativeInfluenceVisual,
    BibleGraphNativeLabelBillboard, BibleGraphNativeLabelOverlayCamera,
    BibleGraphNativeNodeLabelVisual, BibleGraphNativeNodeTextEditorCaret,
    BibleGraphNativeNodeTextEditorText, BibleGraphNativeNodeTextEditorVisual,
    BibleGraphNativeNodeVisual, BibleGraphNativeRenderConfig, BibleGraphNativeRenderPlugin,
    BibleGraphNativeRendererWindowBounds, BibleGraphNativeRendererWindowScene,
    BibleGraphNativeRendererWindowStatus, BibleGraphNativeTextEditorSettings,
    BibleGraphNativeVisualEntity, BibleGraphNativeVisualStatus, BibleGraphNativeWindowControl,
    BibleGraphNativeWindowControlHandle, BibleGraphNativeWindowRunnerConfig,
    BibleGraphNativeWorkspaceTimelineClipVisual, BibleGraphNativeWorkspaceTimelineRoot,
    BibleGraphNativeWorkspaceTimelineVisualEntity, BibleGraphNativeWorkspaceTimelineVisualState,
    configure_controlled_minimal_bible_graph_native_window_app,
    configure_minimal_bible_graph_native_window_app, emit_bible_graph_native_clear_selection,
    emit_bible_graph_native_connected_node_create, emit_bible_graph_native_edge_selection,
    emit_bible_graph_native_influence_selection, emit_bible_graph_native_node_delete,
    emit_bible_graph_native_node_focus, emit_bible_graph_native_node_inspection,
    emit_bible_graph_native_node_name_set, emit_bible_graph_native_node_navigation,
    emit_bible_graph_native_node_selection, native_workspace_timeline_panel_transform,
    rebuild_bible_graph_native_workspace_timeline_visuals,
    run_controlled_minimal_bible_graph_native_window, run_minimal_bible_graph_native_window,
};
pub use scene::{
    BibleGraphEdgeEntity, BibleGraphInfluenceEntity, BibleGraphNodeEntity, BibleGraphSceneStats,
    rebuild_bible_graph_scene,
};
pub use visual::{
    BibleGraphVisualEdge, BibleGraphVisualNode, BibleGraphVisualSnapshot,
    build_bible_graph_visual_snapshot,
};
pub use visual_3d::{
    BibleGraphVisual3dEdge, BibleGraphVisual3dEdgeClass, BibleGraphVisual3dNode,
    BibleGraphVisual3dSnapshot, build_bible_graph_visual_3d_snapshot,
};
pub use workspace::{
    BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_DEPTH, BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_HEIGHT,
    BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_WIDTH, BibleGraphWorkspaceProjection,
    BibleGraphWorkspaceTimelineClipVisual, BibleGraphWorkspaceTimelineSceneStats,
    BibleGraphWorkspaceTimelineTrackVisual, BibleGraphWorkspaceTimelineVisualSnapshot,
};
pub use workspace::{
    BibleGraphWorkspaceTimelineAnchor, BibleGraphWorkspaceTimelinePresentation,
    BibleGraphWorkspaceTimelinePresentationMode,
};

pub const BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY: usize = 128;
pub const BIBLE_GRAPH_FULL_REBUILD_NODE_LIMIT: usize = 500;
pub const BIBLE_GRAPH_FULL_REBUILD_EDGE_LIMIT: usize = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BibleGraphRendererCommand {
    SelectNode {
        node_id: BibleGraphNodeId,
    },
    SelectEdge {
        edge_id: BibleGraphEdgeId,
    },
    SelectInfluence {
        influence_id: ContextInfluenceId,
    },
    InspectNode {
        node_id: BibleGraphNodeId,
    },
    FocusNode {
        node_id: BibleGraphNodeId,
    },
    NavigateToNode {
        node_id: BibleGraphNodeId,
    },
    DeleteNode {
        node_id: BibleGraphNodeId,
    },
    CreateConnectedNode {
        parent_id: BibleGraphNodeId,
    },
    SetNodeName {
        node_id: BibleGraphNodeId,
        name: String,
    },
    SetNodeText {
        node_id: BibleGraphNodeId,
        text: String,
    },
    ClearSelection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BibleGraphCameraCommand {
    FitGraph,
    ResetCamera,
    FrameNode { node_id: BibleGraphNodeId },
    FrameEdge { edge_id: BibleGraphEdgeId },
    FrameInfluence { influence_id: ContextInfluenceId },
    NavigateToNode { node_id: BibleGraphNodeId },
    NavigateToNeighborhood { node_id: BibleGraphNodeId },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BibleGraphRendererError {
    #[error("bible graph projection has not been loaded")]
    MissingProjection,
    #[error("bible graph projection does not contain node {node_id:?}")]
    UnknownNode { node_id: BibleGraphNodeId },
    #[error("bible graph projection does not contain edge {edge_id:?}")]
    UnknownEdge { edge_id: BibleGraphEdgeId },
    #[error("bible graph projection does not contain influence {influence_id:?}")]
    UnknownInfluence { influence_id: ContextInfluenceId },
    #[error(
        "bible graph projection exceeds prototype full-rebuild limits: {node_count} nodes/{edge_count} edges exceeds {node_limit} nodes/{edge_limit} edges"
    )]
    ProjectionExceedsPrototypeRebuildLimit {
        node_count: usize,
        edge_count: usize,
        node_limit: usize,
        edge_limit: usize,
    },
    #[error("bible graph renderer command queue is full")]
    CommandQueueFull,
    #[error("workspace timeline transition progress must be between 0.0 and 1.0")]
    InvalidTimelinePresentationProgress,
}

#[derive(Resource, Default)]
struct BibleGraphRenderState {
    projection: Option<BibleRenderGraphProjection>,
}

#[derive(Resource, Default)]
struct BibleGraphRendererCommandQueue {
    commands: Vec<BibleGraphRendererCommand>,
}

pub struct BibleGraphRendererApp {
    app: App,
}

impl Default for BibleGraphRendererApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BibleGraphRendererApp {
    pub fn new() -> Self {
        let mut app = App::new();
        app.insert_resource(BibleGraphRenderState::default());
        app.insert_resource(BibleGraphRendererCommandQueue::default());
        app.insert_resource(BibleGraphSceneStats::default());
        workspace::insert_workspace_resources(&mut app);
        Self { app }
    }

    #[cfg(feature = "native_render")]
    pub fn new_renderer_window() -> Self {
        let mut renderer = Self::new();
        renderer.app.add_plugins(BibleGraphNativeRenderPlugin);
        renderer.app.update();
        renderer
    }

    #[cfg(feature = "native_render")]
    pub fn renderer_window_ready(&self) -> bool {
        self.app
            .world()
            .get_resource::<BibleGraphNativeRendererWindowStatus>()
            .map(|status| status.camera_count == 2)
            .unwrap_or_default()
    }

    #[cfg(feature = "native_render")]
    pub fn renderer_window_bounds(&self) -> BibleGraphNativeRendererWindowBounds {
        self.app
            .world()
            .get_resource::<BibleGraphNativeRendererWindowStatus>()
            .map(|status| status.bounds)
            .unwrap_or_default()
    }

    #[cfg(feature = "native_render")]
    pub fn set_renderer_window_bounds(&mut self, width_px: u32, height_px: u32) {
        native_render::update_bible_graph_renderer_window_bounds(
            self.app.world_mut(),
            width_px,
            height_px,
        );
    }

    #[cfg(feature = "native_render")]
    pub fn native_visual_counts(&self) -> (usize, usize) {
        self.app
            .world()
            .get_resource::<BibleGraphNativeVisualStatus>()
            .map(|status| (status.node_count, status.edge_count))
            .unwrap_or_default()
    }

    pub fn set_projection(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> Result<(), BibleGraphRendererError> {
        validate_projection_rebuild_limits(&projection)?;
        self.app
            .world_mut()
            .resource_mut::<BibleGraphRenderState>()
            .projection = Some(projection.clone());
        rebuild_bible_graph_scene(self.app.world_mut(), &projection);
        #[cfg(feature = "native_render")]
        native_render::rebuild_bible_graph_native_visuals(self.app.world_mut(), &projection);
        Ok(())
    }

    pub fn projection_node_count(&self) -> usize {
        self.app
            .world()
            .resource::<BibleGraphRenderState>()
            .projection
            .as_ref()
            .map(|projection| projection.nodes.len())
            .unwrap_or_default()
    }

    pub fn scene_counts(&self) -> (usize, usize) {
        let stats = self.app.world().resource::<BibleGraphSceneStats>();
        (stats.node_count, stats.edge_count)
    }

    pub fn influence_count(&self) -> usize {
        self.app
            .world()
            .resource::<BibleGraphSceneStats>()
            .influence_count
    }

    pub fn visual_snapshot(&self) -> Result<BibleGraphVisualSnapshot, BibleGraphRendererError> {
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;
        Ok(build_bible_graph_visual_snapshot(projection))
    }

    pub fn select_node(
        &mut self,
        node_id: BibleGraphNodeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.enqueue_command(BibleGraphRendererCommand::SelectNode { node_id })
    }

    pub fn inspect_node(
        &mut self,
        node_id: BibleGraphNodeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.enqueue_command(BibleGraphRendererCommand::InspectNode { node_id })
    }

    pub fn focus_node(&mut self, node_id: BibleGraphNodeId) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.enqueue_command(BibleGraphRendererCommand::FocusNode { node_id })
    }

    pub fn navigate_to_node(
        &mut self,
        node_id: BibleGraphNodeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.enqueue_command(BibleGraphRendererCommand::NavigateToNode { node_id })
    }

    pub fn delete_node(
        &mut self,
        node_id: BibleGraphNodeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.enqueue_command(BibleGraphRendererCommand::DeleteNode { node_id })
    }

    pub fn create_connected_node(
        &mut self,
        parent_id: BibleGraphNodeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&parent_id)?;
        self.enqueue_command(BibleGraphRendererCommand::CreateConnectedNode { parent_id })
    }

    pub fn set_node_name(
        &mut self,
        node_id: BibleGraphNodeId,
        name: String,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.enqueue_command(BibleGraphRendererCommand::SetNodeName { node_id, name })
    }

    pub fn select_edge(
        &mut self,
        edge_id: BibleGraphEdgeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_edge(&edge_id)?;
        self.enqueue_command(BibleGraphRendererCommand::SelectEdge { edge_id })
    }

    pub fn select_influence(
        &mut self,
        influence_id: ContextInfluenceId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_influence(influence_id)?;
        self.enqueue_command(BibleGraphRendererCommand::SelectInfluence { influence_id })
    }

    pub fn clear_selection(&mut self) -> Result<(), BibleGraphRendererError> {
        self.enqueue_command(BibleGraphRendererCommand::ClearSelection)
    }

    pub fn apply_camera_command(
        &mut self,
        command: BibleGraphCameraCommand,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_camera_command(&command)?;
        #[cfg(feature = "native_render")]
        native_render::apply_bible_graph_native_camera_command(self.app.world_mut(), command);
        Ok(())
    }

    pub fn neighborhood(
        &self,
        node_id: &BibleGraphNodeId,
    ) -> Result<Option<BibleRenderGraphNeighborhood>, BibleGraphRendererError> {
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;
        if !projection.nodes.iter().any(|node| &node.node_id == node_id) {
            return Err(BibleGraphRendererError::UnknownNode {
                node_id: node_id.clone(),
            });
        }
        Ok(projection
            .neighborhoods
            .iter()
            .find(|neighborhood| &neighborhood.node_id == node_id)
            .cloned())
    }

    pub fn edge_ids_for_node(
        &self,
        node_id: &BibleGraphNodeId,
    ) -> Result<Vec<BibleGraphEdgeId>, BibleGraphRendererError> {
        Ok(self
            .neighborhood(node_id)?
            .map(|neighborhood| neighborhood.edge_ids)
            .unwrap_or_default())
    }

    pub fn influence_ids_for_node(
        &self,
        node_id: &BibleGraphNodeId,
    ) -> Result<Vec<ContextInfluenceId>, BibleGraphRendererError> {
        self.validate_node(node_id)?;
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;
        Ok(projection
            .influences
            .iter()
            .filter(|influence| influence.bible_node_id.as_ref() == Some(node_id))
            .map(|influence| influence.influence_id)
            .collect())
    }

    pub fn influence_ids_for_edge(
        &self,
        edge_id: &BibleGraphEdgeId,
    ) -> Result<Vec<ContextInfluenceId>, BibleGraphRendererError> {
        self.validate_edge(edge_id)?;
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;
        Ok(projection
            .influences
            .iter()
            .filter(|influence| influence.bible_edge_id.as_ref() == Some(edge_id))
            .map(|influence| influence.influence_id)
            .collect())
    }

    pub fn drain_commands(&mut self) -> Vec<BibleGraphRendererCommand> {
        std::mem::take(
            &mut self
                .app
                .world_mut()
                .resource_mut::<BibleGraphRendererCommandQueue>()
                .commands,
        )
    }

    fn enqueue_command(
        &mut self,
        command: BibleGraphRendererCommand,
    ) -> Result<(), BibleGraphRendererError> {
        let mut queue = self
            .app
            .world_mut()
            .resource_mut::<BibleGraphRendererCommandQueue>();
        if queue.commands.len() >= BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY {
            return Err(BibleGraphRendererError::CommandQueueFull);
        }

        queue.commands.push(command);
        Ok(())
    }

    fn validate_node(&self, node_id: &BibleGraphNodeId) -> Result<(), BibleGraphRendererError> {
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;

        if projection.nodes.iter().any(|node| &node.node_id == node_id) {
            Ok(())
        } else {
            Err(BibleGraphRendererError::UnknownNode {
                node_id: node_id.clone(),
            })
        }
    }

    fn validate_edge(&self, edge_id: &BibleGraphEdgeId) -> Result<(), BibleGraphRendererError> {
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;

        if projection.edges.iter().any(|edge| &edge.edge_id == edge_id) {
            Ok(())
        } else {
            Err(BibleGraphRendererError::UnknownEdge {
                edge_id: edge_id.clone(),
            })
        }
    }

    fn validate_influence(
        &self,
        influence_id: ContextInfluenceId,
    ) -> Result<(), BibleGraphRendererError> {
        let state = self.app.world().resource::<BibleGraphRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(BibleGraphRendererError::MissingProjection)?;

        if projection
            .influences
            .iter()
            .any(|influence| influence.influence_id == influence_id)
        {
            Ok(())
        } else {
            Err(BibleGraphRendererError::UnknownInfluence { influence_id })
        }
    }

    fn validate_camera_command(
        &self,
        command: &BibleGraphCameraCommand,
    ) -> Result<(), BibleGraphRendererError> {
        match command {
            BibleGraphCameraCommand::FitGraph | BibleGraphCameraCommand::ResetCamera => Ok(()),
            BibleGraphCameraCommand::FrameNode { node_id }
            | BibleGraphCameraCommand::NavigateToNode { node_id }
            | BibleGraphCameraCommand::NavigateToNeighborhood { node_id } => {
                self.validate_node(node_id)
            }
            BibleGraphCameraCommand::FrameEdge { edge_id } => self.validate_edge(edge_id),
            BibleGraphCameraCommand::FrameInfluence { influence_id } => {
                self.validate_influence(*influence_id)
            }
        }
    }
}

fn validate_projection_rebuild_limits(
    projection: &BibleRenderGraphProjection,
) -> Result<(), BibleGraphRendererError> {
    if projection.nodes.len() > BIBLE_GRAPH_FULL_REBUILD_NODE_LIMIT
        || projection.edges.len() > BIBLE_GRAPH_FULL_REBUILD_EDGE_LIMIT
    {
        return Err(
            BibleGraphRendererError::ProjectionExceedsPrototypeRebuildLimit {
                node_count: projection.nodes.len(),
                edge_count: projection.edges.len(),
                node_limit: BIBLE_GRAPH_FULL_REBUILD_NODE_LIMIT,
                edge_limit: BIBLE_GRAPH_FULL_REBUILD_EDGE_LIMIT,
            },
        );
    }

    Ok(())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
