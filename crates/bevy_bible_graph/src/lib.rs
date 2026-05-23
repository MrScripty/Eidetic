use bevy::prelude::{App, Resource};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphNeighborhood, BibleRenderGraphProjection,
    ContextInfluenceId,
};
use serde::Serialize;
use thiserror::Error;

mod scene;
mod visual;

#[cfg(feature = "native_render")]
mod native_render;

#[cfg(feature = "native_render")]
pub use native_render::{
    BibleGraphNativeCamera, BibleGraphNativePanelScene, BibleGraphNativePanelStatus,
    BibleGraphNativeRenderConfig, BibleGraphNativeRenderPlugin,
};
pub use scene::{
    BibleGraphEdgeEntity, BibleGraphInfluenceEntity, BibleGraphNodeEntity, BibleGraphSceneStats,
    rebuild_bible_graph_scene,
};
pub use visual::{
    BibleGraphVisualEdge, BibleGraphVisualNode, BibleGraphVisualSnapshot,
    build_bible_graph_visual_snapshot,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BibleGraphRendererCommand {
    SelectNode { node_id: BibleGraphNodeId },
    SelectEdge { edge_id: BibleGraphEdgeId },
    SelectInfluence { influence_id: ContextInfluenceId },
    InspectNode { node_id: BibleGraphNodeId },
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
        Self { app }
    }

    #[cfg(feature = "native_render")]
    pub fn new_native_panel() -> Self {
        let mut renderer = Self::new();
        renderer.app.add_plugins(BibleGraphNativeRenderPlugin);
        renderer.app.update();
        renderer
    }

    #[cfg(feature = "native_render")]
    pub fn native_panel_ready(&self) -> bool {
        self.app
            .world()
            .get_resource::<BibleGraphNativePanelStatus>()
            .map(|status| status.camera_count == 1)
            .unwrap_or_default()
    }

    pub fn set_projection(&mut self, projection: BibleRenderGraphProjection) {
        self.app
            .world_mut()
            .resource_mut::<BibleGraphRenderState>()
            .projection = Some(projection.clone());
        rebuild_bible_graph_scene(self.app.world_mut(), &projection);
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
        self.app
            .world_mut()
            .resource_mut::<BibleGraphRendererCommandQueue>()
            .commands
            .push(BibleGraphRendererCommand::SelectNode { node_id });
        Ok(())
    }

    pub fn inspect_node(
        &mut self,
        node_id: BibleGraphNodeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_node(&node_id)?;
        self.app
            .world_mut()
            .resource_mut::<BibleGraphRendererCommandQueue>()
            .commands
            .push(BibleGraphRendererCommand::InspectNode { node_id });
        Ok(())
    }

    pub fn select_edge(
        &mut self,
        edge_id: BibleGraphEdgeId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_edge(&edge_id)?;
        self.app
            .world_mut()
            .resource_mut::<BibleGraphRendererCommandQueue>()
            .commands
            .push(BibleGraphRendererCommand::SelectEdge { edge_id });
        Ok(())
    }

    pub fn select_influence(
        &mut self,
        influence_id: ContextInfluenceId,
    ) -> Result<(), BibleGraphRendererError> {
        self.validate_influence(influence_id)?;
        self.app
            .world_mut()
            .resource_mut::<BibleGraphRendererCommandQueue>()
            .commands
            .push(BibleGraphRendererCommand::SelectInfluence { influence_id });
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
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
