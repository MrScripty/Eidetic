use bevy::prelude::{App, Resource};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphNeighborhood, BibleRenderGraphProjection,
};
use serde::Serialize;
use thiserror::Error;

mod scene;

pub use scene::{
    BibleGraphEdgeEntity, BibleGraphNodeEntity, BibleGraphSceneStats, rebuild_bible_graph_scene,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BibleGraphRendererCommand {
    SelectNode { node_id: BibleGraphNodeId },
    InspectNode { node_id: BibleGraphNodeId },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BibleGraphRendererError {
    #[error("bible graph projection has not been loaded")]
    MissingProjection,
    #[error("bible graph projection does not contain node {node_id:?}")]
    UnknownNode { node_id: BibleGraphNodeId },
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        BibleGraphEdgeKind, BibleGraphSchemaKey, BibleRenderGraphEdge, BibleRenderGraphNode,
        BibleRenderGraphPosition,
    };

    #[test]
    fn renderer_app_receives_projection_and_emits_validated_selection_command() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let mut renderer = BibleGraphRendererApp::new();

        renderer.set_projection(projection_with_node(node_id.clone()));

        assert_eq!(renderer.projection_node_count(), 1);
        assert_eq!(renderer.select_node(node_id.clone()), Ok(()));
        assert_eq!(
            renderer.drain_commands(),
            vec![BibleGraphRendererCommand::SelectNode { node_id }]
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_rebuilds_scene_entities_from_projection() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let mut renderer = BibleGraphRendererApp::new();

        renderer.set_projection(projection_with_edge(node_id));
        assert_eq!(renderer.scene_counts(), (2, 1));

        renderer.set_projection(BibleRenderGraphProjection {
            nodes: Vec::new(),
            edges: Vec::new(),
            neighborhoods: Vec::new(),
        });
        assert_eq!(renderer.scene_counts(), (0, 0));
    }

    #[test]
    fn renderer_app_rejects_selection_before_projection_load() {
        let mut renderer = BibleGraphRendererApp::new();
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();

        assert_eq!(
            renderer.select_node(node_id),
            Err(BibleGraphRendererError::MissingProjection)
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_rejects_unknown_node_selection() {
        let mut renderer = BibleGraphRendererApp::new();
        let known_node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let unknown_node_id = BibleGraphNodeId::new("node.character.nope").unwrap();
        renderer.set_projection(projection_with_node(known_node_id));

        assert_eq!(
            renderer.inspect_node(unknown_node_id.clone()),
            Err(BibleGraphRendererError::UnknownNode {
                node_id: unknown_node_id
            })
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_returns_neighborhood_indexes_from_projection() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let mut renderer = BibleGraphRendererApp::new();
        renderer.set_projection(projection_with_edge(node_id.clone()));

        assert_eq!(
            renderer.edge_ids_for_node(&node_id),
            Ok(vec![BibleGraphEdgeId::new("edge.ada.beach").unwrap()])
        );
    }

    fn projection_with_node(node_id: BibleGraphNodeId) -> BibleRenderGraphProjection {
        BibleRenderGraphProjection {
            nodes: vec![BibleRenderGraphNode {
                node_id,
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                label: "Ada".to_string(),
                system_owned: false,
                sort_order: 0,
                depth: 0,
                position: BibleRenderGraphPosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            }],
            edges: Vec::new(),
            neighborhoods: Vec::new(),
        }
    }

    fn projection_with_edge(source_id: BibleGraphNodeId) -> BibleRenderGraphProjection {
        let target_id = BibleGraphNodeId::new("node.place.beach").unwrap();
        let edge_id = BibleGraphEdgeId::new("edge.ada.beach").unwrap();
        BibleRenderGraphProjection {
            nodes: vec![
                BibleRenderGraphNode {
                    node_id: source_id.clone(),
                    parent_id: None,
                    schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                    label: "Ada".to_string(),
                    system_owned: false,
                    sort_order: 0,
                    depth: 0,
                    position: BibleRenderGraphPosition {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                BibleRenderGraphNode {
                    node_id: target_id.clone(),
                    parent_id: None,
                    schema_key: BibleGraphSchemaKey::new("place").unwrap(),
                    label: "Beach".to_string(),
                    system_owned: false,
                    sort_order: 1,
                    depth: 0,
                    position: BibleRenderGraphPosition {
                        x: 0.0,
                        y: 150.0,
                        z: 0.0,
                    },
                },
            ],
            edges: vec![BibleRenderGraphEdge {
                edge_id: edge_id.clone(),
                from_node_id: source_id.clone(),
                to_node_id: target_id.clone(),
                edge_kind: BibleGraphEdgeKind::LocatedIn,
                label: "located in".to_string(),
                directed: true,
                sort_order: 0,
            }],
            neighborhoods: vec![
                BibleRenderGraphNeighborhood {
                    node_id: source_id,
                    connected_node_ids: vec![target_id.clone()],
                    edge_ids: vec![edge_id.clone()],
                },
                BibleRenderGraphNeighborhood {
                    node_id: target_id,
                    connected_node_ids: Vec::new(),
                    edge_ids: vec![edge_id],
                },
            ],
        }
    }
}
