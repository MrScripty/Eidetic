use bevy::prelude::Resource;
use eidetic_core::contracts::{BibleRenderGraphProjection, TimelineRenderProjection};
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;
use serde::{Deserialize, Serialize};

use crate::{BibleGraphRendererApp, BibleGraphRendererError};

pub const BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_WIDTH: f32 = 760.0;
pub const BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_HEIGHT: f32 = 150.0;
pub const BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_DEPTH: f32 = 4.0;
const WORKSPACE_TIMELINE_HORIZONTAL_PADDING: f32 = 24.0;
const WORKSPACE_TIMELINE_TOP_PADDING: f32 = 22.0;
const WORKSPACE_TIMELINE_CLIP_HEIGHT: f32 = 16.0;
const WORKSPACE_TIMELINE_TRACK_GAP: f32 = 7.0;

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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphWorkspaceTimelineVisualSnapshot {
    pub panel_width: f32,
    pub panel_height: f32,
    pub tracks: Vec<BibleGraphWorkspaceTimelineTrackVisual>,
    pub clips: Vec<BibleGraphWorkspaceTimelineClipVisual>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphWorkspaceTimelineTrackVisual {
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub label: String,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphWorkspaceTimelineClipVisual {
    pub node_id: NodeId,
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub selected: bool,
    pub color_rgb: [f32; 3],
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
            viewport_offset_y: -0.22,
        }
    }
}

#[derive(Resource, Default)]
struct BibleGraphWorkspaceTimelineState {
    projection: Option<TimelineRenderProjection>,
    scene_stats: BibleGraphWorkspaceTimelineSceneStats,
    visual_snapshot: BibleGraphWorkspaceTimelineVisualSnapshot,
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

    pub fn workspace_timeline_visual_snapshot(&self) -> BibleGraphWorkspaceTimelineVisualSnapshot {
        self.app
            .world()
            .resource::<BibleGraphWorkspaceTimelineState>()
            .visual_snapshot
            .clone()
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
        let visual_snapshot = projection
            .as_ref()
            .map(build_workspace_timeline_visual_snapshot)
            .unwrap_or_default();
        let mut timeline_state = self
            .app
            .world_mut()
            .resource_mut::<BibleGraphWorkspaceTimelineState>();
        timeline_state.projection = projection;
        timeline_state.scene_stats = scene_stats;
        timeline_state.visual_snapshot = visual_snapshot;
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

fn build_workspace_timeline_visual_snapshot(
    projection: &TimelineRenderProjection,
) -> BibleGraphWorkspaceTimelineVisualSnapshot {
    let panel_width = BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_WIDTH;
    let panel_height = BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_HEIGHT;
    let usable_width = panel_width - (WORKSPACE_TIMELINE_HORIZONTAL_PADDING * 2.0);
    let duration_ms = projection.total_duration_ms.max(1);
    let mut tracks = projection.tracks.clone();
    tracks.sort_by_key(|track| track.sort_order);

    let track_visuals = tracks
        .iter()
        .enumerate()
        .map(
            |(track_index, track)| BibleGraphWorkspaceTimelineTrackVisual {
                track_id: track.track_id,
                level: track.level,
                label: track.label.clone(),
                y: timeline_track_y(track_index, panel_height),
            },
        )
        .collect::<Vec<_>>();

    let clips = projection
        .clips
        .iter()
        .filter_map(|clip| {
            let track_index = tracks
                .iter()
                .position(|track| track.track_id == clip.track_id)?;
            let start_ratio = clip.start_ms as f32 / duration_ms as f32;
            let end_ratio = clip.end_ms.min(duration_ms) as f32 / duration_ms as f32;
            let width = ((end_ratio - start_ratio).max(0.0) * usable_width).max(1.0);
            let x_start = -(panel_width / 2.0)
                + WORKSPACE_TIMELINE_HORIZONTAL_PADDING
                + (start_ratio * usable_width);
            let selected = projection.selected_node_id.as_ref() == Some(&clip.node_id);
            Some(BibleGraphWorkspaceTimelineClipVisual {
                node_id: clip.node_id,
                track_id: clip.track_id,
                level: clip.level,
                label: clip.name.clone(),
                x: x_start + (width / 2.0),
                y: timeline_track_y(track_index, panel_height),
                width,
                height: if selected {
                    WORKSPACE_TIMELINE_CLIP_HEIGHT + 4.0
                } else {
                    WORKSPACE_TIMELINE_CLIP_HEIGHT
                },
                start_ms: clip.start_ms,
                end_ms: clip.end_ms,
                selected,
                color_rgb: workspace_timeline_clip_color_rgb(
                    clip.level,
                    clip.locked,
                    clip.content_status,
                    selected,
                ),
            })
        })
        .collect::<Vec<_>>();

    BibleGraphWorkspaceTimelineVisualSnapshot {
        panel_width,
        panel_height,
        tracks: track_visuals,
        clips,
    }
}

fn timeline_track_y(track_index: usize, panel_height: f32) -> f32 {
    (panel_height / 2.0)
        - WORKSPACE_TIMELINE_TOP_PADDING
        - (track_index as f32 * (WORKSPACE_TIMELINE_CLIP_HEIGHT + WORKSPACE_TIMELINE_TRACK_GAP))
}

fn workspace_timeline_clip_color_rgb(
    level: StoryLevel,
    locked: bool,
    content_status: ContentStatus,
    selected: bool,
) -> [f32; 3] {
    if selected {
        return [0.957, 0.769, 0.188];
    }
    if locked {
        return [0.431, 0.455, 0.502];
    }
    match content_status {
        ContentStatus::Generating => [0.937, 0.706, 0.294],
        ContentStatus::HasContent => [0.282, 0.686, 0.424],
        ContentStatus::NotesOnly => match level {
            StoryLevel::Premise => [0.576, 0.412, 0.776],
            StoryLevel::Act => [0.518, 0.553, 0.859],
            StoryLevel::Sequence => [0.376, 0.592, 0.827],
            StoryLevel::Scene => [0.342, 0.655, 0.691],
            StoryLevel::Beat => [0.451, 0.714, 0.455],
        },
        ContentStatus::Empty => [0.188, 0.227, 0.298],
    }
}
