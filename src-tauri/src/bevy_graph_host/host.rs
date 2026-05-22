use std::panic::{AssertUnwindSafe, catch_unwind};

use eidetic_bevy_bible_graph::{BibleGraphRendererApp, BibleGraphRendererError};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};

use super::{BibleGraphHostError, BibleGraphHostStatus};

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

    pub fn select_edge(&mut self, edge_id: BibleGraphEdgeId) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_edge(edge_id))
    }

    pub fn select_influence(
        &mut self,
        influence_id: ContextInfluenceId,
    ) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_influence(influence_id))
    }

    pub fn drain_commands(&mut self) -> Vec<eidetic_bevy_bible_graph::BibleGraphRendererCommand> {
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
