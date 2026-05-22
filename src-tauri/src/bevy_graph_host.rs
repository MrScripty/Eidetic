use std::panic::{AssertUnwindSafe, catch_unwind};

use eidetic_bevy_bible_graph::{
    BibleGraphRendererApp, BibleGraphRendererCommand, BibleGraphRendererError,
};
use eidetic_core::contracts::{BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleGraphHostStatus {
    pub running: bool,
    pub node_count: usize,
    pub edge_count: usize,
    pub influence_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BibleGraphHostError {
    Renderer(String),
    RendererPanic,
}

pub struct DesktopBibleGraphHost {
    renderer: Option<BibleGraphRendererApp>,
    last_error: Option<String>,
}

impl Default for DesktopBibleGraphHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopBibleGraphHost {
    pub fn new() -> Self {
        Self {
            renderer: None,
            last_error: None,
        }
    }

    pub fn start(&mut self) -> Result<BibleGraphHostStatus, BibleGraphHostError> {
        if self.renderer.is_none() {
            self.renderer = Some(Self::catch_renderer_panic(BibleGraphRendererApp::new)?);
        }
        self.last_error = None;
        Ok(self.status())
    }

    pub fn stop(&mut self) -> BibleGraphHostStatus {
        self.renderer = None;
        self.last_error = None;
        self.status()
    }

    pub fn set_projection(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> Result<BibleGraphHostStatus, BibleGraphHostError> {
        self.start()?;
        self.with_renderer_mut(|renderer| {
            renderer.set_projection(projection);
            Ok(())
        })?;
        Ok(self.status())
    }

    pub fn select_node(&mut self, node_id: BibleGraphNodeId) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_node(node_id))
    }

    pub fn inspect_node(&mut self, node_id: BibleGraphNodeId) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.inspect_node(node_id))
    }

    pub fn select_influence(
        &mut self,
        influence_id: ContextInfluenceId,
    ) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_influence(influence_id))
    }

    pub fn drain_commands(&mut self) -> Vec<BibleGraphRendererCommand> {
        self.renderer
            .as_mut()
            .map(BibleGraphRendererApp::drain_commands)
            .unwrap_or_default()
    }

    pub fn status(&self) -> BibleGraphHostStatus {
        let (node_count, edge_count, influence_count) = self
            .renderer
            .as_ref()
            .map(|renderer| {
                let (node_count, edge_count) = renderer.scene_counts();
                (node_count, edge_count, renderer.influence_count())
            })
            .unwrap_or_default();

        BibleGraphHostStatus {
            running: self.renderer.is_some(),
            node_count,
            edge_count,
            influence_count,
            last_error: self.last_error.clone(),
        }
    }

    fn with_renderer_mut<T>(
        &mut self,
        operation: impl FnOnce(&mut BibleGraphRendererApp) -> Result<T, BibleGraphRendererError>,
    ) -> Result<T, BibleGraphHostError> {
        self.start()?;
        let renderer = self.renderer.as_mut().expect("renderer is started");
        let result = Self::catch_renderer_panic(|| operation(renderer))?.map_err(|error| {
            let message = error.to_string();
            self.last_error = Some(message.clone());
            BibleGraphHostError::Renderer(message)
        });
        if result.is_ok() {
            self.last_error = None;
        }
        result
    }

    fn catch_renderer_panic<T>(operation: impl FnOnce() -> T) -> Result<T, BibleGraphHostError> {
        catch_unwind(AssertUnwindSafe(operation)).map_err(|_| BibleGraphHostError::RendererPanic)
    }
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::{
        BibleGraphEdgeKind, BibleRenderGraphEdge, BibleRenderGraphInfluence,
        BibleRenderGraphNeighborhood, BibleRenderGraphNode, BibleRenderGraphPosition,
        ContextInfluenceKind, ContextInfluenceProvenance,
    };
    use eidetic_core::timeline::node::StoryLevel;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn host_applies_projection_and_reports_scene_counts() {
        let mut host = DesktopBibleGraphHost::new();

        let status = host.set_projection(sample_projection()).unwrap();

        assert_eq!(
            status,
            BibleGraphHostStatus {
                running: true,
                node_count: 2,
                edge_count: 1,
                influence_count: 1,
                last_error: None,
            }
        );
    }

    #[test]
    fn host_validates_renderer_commands_and_drains_them() {
        let mut host = DesktopBibleGraphHost::new();
        let projection = sample_projection();
        let node_id = projection.nodes[0].node_id.clone();
        let influence_id = projection.influences[0].influence_id;
        host.set_projection(projection).unwrap();

        host.select_node(node_id.clone()).unwrap();
        host.inspect_node(node_id.clone()).unwrap();
        host.select_influence(influence_id).unwrap();

        assert_eq!(
            host.drain_commands(),
            vec![
                BibleGraphRendererCommand::SelectNode {
                    node_id: node_id.clone()
                },
                BibleGraphRendererCommand::InspectNode { node_id },
                BibleGraphRendererCommand::SelectInfluence { influence_id },
            ]
        );
        assert!(host.drain_commands().is_empty());
    }

    #[test]
    fn host_records_renderer_errors_without_panicking() {
        let mut host = DesktopBibleGraphHost::new();
        host.set_projection(sample_projection()).unwrap();
        let missing = BibleGraphNodeId::new("node.missing").unwrap();

        let error = host.select_node(missing).unwrap_err();

        assert!(matches!(error, BibleGraphHostError::Renderer(_)));
        assert!(host.status().last_error.is_some());
    }

    #[test]
    fn host_stop_drops_renderer_state() {
        let mut host = DesktopBibleGraphHost::new();
        host.set_projection(sample_projection()).unwrap();

        let status = host.stop();

        assert_eq!(
            status,
            BibleGraphHostStatus {
                running: false,
                node_count: 0,
                edge_count: 0,
                influence_count: 0,
                last_error: None,
            }
        );
    }

    fn sample_projection() -> BibleRenderGraphProjection {
        let ada_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let beach_id = BibleGraphNodeId::new("node.location.beach").unwrap();
        let edge_id = eidetic_core::contracts::BibleGraphEdgeId::new("edge.ada.beach").unwrap();
        let influence_id = ContextInfluenceId(Uuid::from_u128(1));
        let timeline_node_id = eidetic_core::timeline::node::NodeId(Uuid::from_u128(2));

        BibleRenderGraphProjection {
            nodes: vec![
                BibleRenderGraphNode {
                    node_id: ada_id.clone(),
                    parent_id: None,
                    schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("character")
                        .unwrap(),
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
                    node_id: beach_id.clone(),
                    parent_id: None,
                    schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new("location")
                        .unwrap(),
                    label: "Beach".to_string(),
                    system_owned: false,
                    sort_order: 1,
                    depth: 0,
                    position: BibleRenderGraphPosition {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
            ],
            edges: vec![BibleRenderGraphEdge {
                edge_id: edge_id.clone(),
                from_node_id: ada_id.clone(),
                to_node_id: beach_id.clone(),
                edge_kind: BibleGraphEdgeKind::LocatedIn,
                label: "located in".to_string(),
                directed: true,
                sort_order: 0,
            }],
            neighborhoods: vec![BibleRenderGraphNeighborhood {
                node_id: ada_id.clone(),
                connected_node_ids: vec![beach_id],
                edge_ids: vec![edge_id.clone()],
            }],
            influences: vec![BibleRenderGraphInfluence {
                influence_id,
                timeline_node_id,
                source_layer: StoryLevel::Scene,
                influence_kind: ContextInfluenceKind::Direct,
                confidence: 0.9,
                reason: "Scene uses Ada at the beach.".to_string(),
                provenance: ContextInfluenceProvenance::AiSelected,
                bible_node_id: Some(ada_id),
                bible_edge_id: Some(edge_id),
                sort_order: 0,
            }],
        }
    }
}
