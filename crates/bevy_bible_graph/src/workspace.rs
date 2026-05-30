use bevy::prelude::Resource;
use eidetic_core::contracts::{BibleRenderGraphProjection, TimelineRenderProjection};
use serde::{Deserialize, Serialize};

use crate::{BibleGraphRendererApp, BibleGraphRendererError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphWorkspaceProjection {
    pub graph: BibleRenderGraphProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<TimelineRenderProjection>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphWorkspaceTimelineSceneStats {
    pub track_count: usize,
    pub clip_count: usize,
    pub relationship_count: usize,
    pub affect_overlay_count: usize,
    pub total_duration_ms: u64,
}

#[derive(Resource, Default)]
struct BibleGraphWorkspaceTimelineState {
    projection: Option<TimelineRenderProjection>,
    scene_stats: BibleGraphWorkspaceTimelineSceneStats,
}

pub(crate) fn insert_workspace_resources(app: &mut bevy::prelude::App) {
    app.insert_resource(BibleGraphWorkspaceTimelineState::default());
}

impl BibleGraphRendererApp {
    pub fn set_workspace_projection(
        &mut self,
        projection: BibleGraphWorkspaceProjection,
    ) -> Result<(), BibleGraphRendererError> {
        self.set_projection(projection.graph)?;
        self.set_workspace_timeline_projection(projection.timeline);
        Ok(())
    }

    pub fn workspace_timeline_scene_stats(&self) -> BibleGraphWorkspaceTimelineSceneStats {
        self.app
            .world()
            .resource::<BibleGraphWorkspaceTimelineState>()
            .scene_stats
    }

    pub fn workspace_has_timeline_projection(&self) -> bool {
        self.app
            .world()
            .resource::<BibleGraphWorkspaceTimelineState>()
            .projection
            .is_some()
    }

    fn set_workspace_timeline_projection(&mut self, projection: Option<TimelineRenderProjection>) {
        let scene_stats = projection
            .as_ref()
            .map(timeline_scene_stats)
            .unwrap_or_default();
        let mut timeline_state = self
            .app
            .world_mut()
            .resource_mut::<BibleGraphWorkspaceTimelineState>();
        timeline_state.projection = projection;
        timeline_state.scene_stats = scene_stats;
    }
}

fn timeline_scene_stats(
    projection: &TimelineRenderProjection,
) -> BibleGraphWorkspaceTimelineSceneStats {
    BibleGraphWorkspaceTimelineSceneStats {
        track_count: projection.tracks.len(),
        clip_count: projection.clips.len(),
        relationship_count: projection.relationships.len(),
        affect_overlay_count: projection.affect_overlays.len(),
        total_duration_ms: projection.total_duration_ms,
    }
}
