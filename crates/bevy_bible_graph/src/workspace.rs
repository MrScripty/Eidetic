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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphWorkspaceTimelineAnchor {
    #[default]
    CameraAnchoredPanel,
    WorldAnchoredTimeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BibleGraphWorkspaceTimelinePresentationMode {
    CameraAnchoredPanel,
    WorldAnchoredTimeline,
    Transitioning {
        from: BibleGraphWorkspaceTimelineAnchor,
        to: BibleGraphWorkspaceTimelineAnchor,
        progress: f32,
    },
}

impl Default for BibleGraphWorkspaceTimelinePresentationMode {
    fn default() -> Self {
        Self::CameraAnchoredPanel
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Resource)]
pub struct BibleGraphWorkspaceTimelinePresentation {
    pub mode: BibleGraphWorkspaceTimelinePresentationMode,
    pub camera_distance: f32,
    pub viewport_offset_x: f32,
    pub viewport_offset_y: f32,
}

impl Default for BibleGraphWorkspaceTimelinePresentation {
    fn default() -> Self {
        Self {
            mode: BibleGraphWorkspaceTimelinePresentationMode::CameraAnchoredPanel,
            camera_distance: 420.0,
            viewport_offset_x: 0.0,
            viewport_offset_y: -0.62,
        }
    }
}

#[derive(Resource, Default)]
struct BibleGraphWorkspaceTimelineState {
    projection: Option<TimelineRenderProjection>,
    scene_stats: BibleGraphWorkspaceTimelineSceneStats,
}

pub(crate) fn insert_workspace_resources(app: &mut bevy::prelude::App) {
    app.insert_resource(BibleGraphWorkspaceTimelineState::default());
    app.insert_resource(BibleGraphWorkspaceTimelinePresentation::default());
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

    pub fn workspace_timeline_presentation(&self) -> BibleGraphWorkspaceTimelinePresentation {
        *self
            .app
            .world()
            .resource::<BibleGraphWorkspaceTimelinePresentation>()
    }

    pub fn set_workspace_timeline_presentation_mode(
        &mut self,
        mode: BibleGraphWorkspaceTimelinePresentationMode,
    ) -> Result<(), BibleGraphRendererError> {
        validate_timeline_presentation_mode(mode)?;
        self.app
            .world_mut()
            .resource_mut::<BibleGraphWorkspaceTimelinePresentation>()
            .mode = mode;
        Ok(())
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

fn validate_timeline_presentation_mode(
    mode: BibleGraphWorkspaceTimelinePresentationMode,
) -> Result<(), BibleGraphRendererError> {
    match mode {
        BibleGraphWorkspaceTimelinePresentationMode::Transitioning { progress, .. }
            if !(0.0..=1.0).contains(&progress) =>
        {
            Err(BibleGraphRendererError::InvalidTimelinePresentationProgress)
        }
        _ => Ok(()),
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
