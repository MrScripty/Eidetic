use bevy::prelude::{
    Camera2d, Color, Commands, Component, Entity, Quat, Resource, Sprite, Transform, Vec2, Vec3,
    World,
};
use eidetic_core::contracts::{
    AffectValueId, TimelineRenderAffectSample, TimelineRenderProjection, TimelineRenderTrack,
};
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use eidetic_core::timeline::track::TrackId;

use crate::native_render::TimelineNativeProjectionState;
use crate::native_style::{
    native_affect_color_rgb, native_affect_height_px, native_clip_color_rgb,
    native_relationship_color_rgb,
};
use crate::scene::TimelineSceneEntity;
use crate::{TimelinePlayhead, TimelineViewport};

const TIMELINE_NATIVE_CLIP_HEIGHT_PX: f32 = 42.0;
const TIMELINE_NATIVE_TRACK_GAP_PX: f32 = 16.0;
const TIMELINE_NATIVE_HORIZONTAL_PADDING_PX: f32 = 48.0;
const TIMELINE_NATIVE_TOP_PADDING_PX: f32 = 48.0;
const TIMELINE_NATIVE_PLAYHEAD_WIDTH_PX: f32 = 3.0;
const TIMELINE_NATIVE_RELATIONSHIP_WIDTH_PX: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct TimelineNativeRenderLayout {
    pub width_px: f32,
    pub height_px: f32,
    pub clip_height_px: f32,
    pub track_gap_px: f32,
    pub horizontal_padding_px: f32,
    pub top_padding_px: f32,
}

#[derive(Component)]
pub struct TimelineNativeVisualEntity;

#[derive(Component)]
pub struct TimelineNativeCamera;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TimelineNativeClipVisual {
    pub node_id: NodeId,
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub locked: bool,
    pub content_status: ContentStatus,
    pub selected: bool,
    pub color_rgb: [f32; 3],
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TimelineNativePlayheadVisual {
    pub position_ms: u64,
    pub x_px: f32,
    pub height_px: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TimelineNativeRelationshipVisual {
    pub relationship_id: RelationshipId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub relationship_type: RelationshipType,
    pub start_px: [f32; 2],
    pub end_px: [f32; 2],
    pub length_px: f32,
    pub color_rgb: [f32; 3],
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TimelineNativeAffectOverlayVisual {
    pub affect_id: AffectValueId,
    pub node_id: NodeId,
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub color_rgb: [f32; 3],
}

impl TimelineNativeRenderLayout {
    pub fn from_window(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px: width_px.max(1) as f32,
            height_px: height_px.max(1) as f32,
            clip_height_px: TIMELINE_NATIVE_CLIP_HEIGHT_PX,
            track_gap_px: TIMELINE_NATIVE_TRACK_GAP_PX,
            horizontal_padding_px: TIMELINE_NATIVE_HORIZONTAL_PADDING_PX,
            top_padding_px: TIMELINE_NATIVE_TOP_PADDING_PX,
        }
    }

    fn usable_width_px(self) -> f32 {
        (self.width_px - (self.horizontal_padding_px * 2.0)).max(1.0)
    }

    fn left_px(self) -> f32 {
        -(self.width_px / 2.0) + self.horizontal_padding_px
    }

    fn top_px(self) -> f32 {
        (self.height_px / 2.0) - self.top_padding_px
    }
}

pub(crate) fn spawn_timeline_native_camera(mut commands: Commands) {
    commands.spawn((Camera2d, TimelineNativeCamera));
}

pub(crate) fn native_track_height_px() -> u32 {
    (TIMELINE_NATIVE_CLIP_HEIGHT_PX + TIMELINE_NATIVE_TRACK_GAP_PX) as u32
}

pub(crate) fn rebuild_timeline_native_visuals(
    world: &mut World,
    projection: &TimelineRenderProjection,
) {
    despawn_existing_timeline_native_visuals(world);

    let layout = world
        .get_resource::<TimelineNativeRenderLayout>()
        .copied()
        .unwrap_or_else(|| TimelineNativeRenderLayout::from_window(1280, 360));
    let viewport = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.viewport)
        .unwrap_or_else(|| TimelineViewport::from_duration(projection.total_duration_ms));
    let viewport_width_ms = viewport.width_ms();
    let sorted_tracks = {
        let mut tracks = projection.tracks.clone();
        tracks.sort_by_key(|track| track.sort_order);
        tracks
    };

    for clip in &projection.clips {
        let Some(track_index) = sorted_tracks
            .iter()
            .position(|track| track.track_id == clip.track_id)
        else {
            continue;
        };
        let visible_start_ms = clip.start_ms.max(viewport.start_ms);
        let visible_end_ms = clip.end_ms.min(viewport.end_ms);
        if visible_end_ms <= visible_start_ms {
            continue;
        }
        let start_ratio =
            visible_start_ms.saturating_sub(viewport.start_ms) as f32 / viewport_width_ms as f32;
        let end_ratio =
            visible_end_ms.saturating_sub(viewport.start_ms) as f32 / viewport_width_ms as f32;
        if end_ratio <= start_ratio {
            continue;
        }
        let x_start = layout.left_px() + (start_ratio * layout.usable_width_px());
        let x_end = layout.left_px() + (end_ratio * layout.usable_width_px());
        let width_px = (x_end - x_start).max(1.0);
        let x_px = x_start + (width_px / 2.0);
        let y_px =
            layout.top_px() - (track_index as f32 * (layout.clip_height_px + layout.track_gap_px));
        let selected = projection.selected_node_id == Some(clip.node_id);
        let color_rgb =
            native_clip_color_rgb(clip.level, clip.locked, clip.content_status, selected);
        let height_px = if selected {
            layout.clip_height_px + 6.0
        } else {
            layout.clip_height_px
        };

        world.spawn((
            TimelineSceneEntity,
            TimelineNativeVisualEntity,
            TimelineNativeClipVisual {
                node_id: clip.node_id,
                track_id: clip.track_id,
                level: clip.level,
                x_px,
                y_px,
                width_px,
                height_px,
                start_ms: clip.start_ms,
                end_ms: clip.end_ms,
                locked: clip.locked,
                content_status: clip.content_status,
                selected,
                color_rgb,
            },
            Sprite::from_color(
                Color::srgb(color_rgb[0], color_rgb[1], color_rgb[2]),
                Vec2::new(width_px, height_px),
            ),
            Transform::from_translation(Vec3::new(x_px, y_px, 0.0)),
        ));
    }

    spawn_timeline_native_relationship_visuals(world, projection, layout, viewport, &sorted_tracks);
    spawn_timeline_native_affect_overlay_visuals(
        world,
        projection,
        layout,
        viewport,
        &sorted_tracks,
    );

    if let Some(playhead) = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.playhead)
    {
        spawn_timeline_native_playhead_visual(world, layout, viewport, playhead);
    }
}

fn spawn_timeline_native_affect_overlay_visuals(
    world: &mut World,
    projection: &TimelineRenderProjection,
    layout: TimelineNativeRenderLayout,
    viewport: TimelineViewport,
    sorted_tracks: &[TimelineRenderTrack],
) {
    for affect in &projection.affect_overlays {
        let Some((x_px, y_px, width_px, height_px)) =
            affect_overlay_geometry_px(projection, sorted_tracks, layout, viewport, affect)
        else {
            continue;
        };
        let color_rgb = native_affect_color_rgb(affect.valence);

        world.spawn((
            TimelineSceneEntity,
            TimelineNativeVisualEntity,
            TimelineNativeAffectOverlayVisual {
                affect_id: affect.affect_id,
                node_id: affect.node_id,
                x_px,
                y_px,
                width_px,
                height_px,
                start_ms: affect.start_ms,
                end_ms: affect.end_ms,
                color_rgb,
            },
            Sprite::from_color(
                Color::srgb(color_rgb[0], color_rgb[1], color_rgb[2]),
                Vec2::new(width_px, height_px),
            ),
            Transform::from_translation(Vec3::new(x_px, y_px, 0.75)),
        ));
    }
}

fn affect_overlay_geometry_px(
    projection: &TimelineRenderProjection,
    sorted_tracks: &[TimelineRenderTrack],
    layout: TimelineNativeRenderLayout,
    viewport: TimelineViewport,
    affect: &TimelineRenderAffectSample,
) -> Option<(f32, f32, f32, f32)> {
    let clip = projection
        .clips
        .iter()
        .find(|clip| clip.node_id == affect.node_id)?;
    let visible_start_ms = affect.start_ms.max(viewport.start_ms);
    let visible_end_ms = affect.end_ms.min(viewport.end_ms);
    if visible_end_ms <= visible_start_ms {
        return None;
    }
    let track_index = sorted_tracks
        .iter()
        .position(|track| track.track_id == clip.track_id)?;
    let start_ratio =
        visible_start_ms.saturating_sub(viewport.start_ms) as f32 / viewport.width_ms() as f32;
    let end_ratio =
        visible_end_ms.saturating_sub(viewport.start_ms) as f32 / viewport.width_ms() as f32;
    if end_ratio <= start_ratio {
        return None;
    }
    let x_start = layout.left_px() + (start_ratio * layout.usable_width_px());
    let x_end = layout.left_px() + (end_ratio * layout.usable_width_px());
    let width_px = (x_end - x_start).max(1.0);
    let x_px = x_start + (width_px / 2.0);
    let clip_y_px =
        layout.top_px() - (track_index as f32 * (layout.clip_height_px + layout.track_gap_px));
    let height_px = native_affect_height_px(affect.intensity);
    let y_px = clip_y_px - (layout.clip_height_px / 2.0) - (height_px / 2.0) - 4.0;
    Some((x_px, y_px, width_px, height_px))
}

fn spawn_timeline_native_relationship_visuals(
    world: &mut World,
    projection: &TimelineRenderProjection,
    layout: TimelineNativeRenderLayout,
    viewport: TimelineViewport,
    sorted_tracks: &[TimelineRenderTrack],
) {
    for relationship in &projection.relationships {
        let Some(start_px) = relationship_endpoint_px(
            projection,
            sorted_tracks,
            layout,
            viewport,
            relationship.from_node_id,
        ) else {
            continue;
        };
        let Some(end_px) = relationship_endpoint_px(
            projection,
            sorted_tracks,
            layout,
            viewport,
            relationship.to_node_id,
        ) else {
            continue;
        };
        let delta_x = end_px[0] - start_px[0];
        let delta_y = end_px[1] - start_px[1];
        let length_px = (delta_x.mul_add(delta_x, delta_y * delta_y))
            .sqrt()
            .max(1.0);
        let center_x = start_px[0] + (delta_x / 2.0);
        let center_y = start_px[1] + (delta_y / 2.0);
        let angle = delta_y.atan2(delta_x);
        let color_rgb = native_relationship_color_rgb(&relationship.relationship_type);

        world.spawn((
            TimelineSceneEntity,
            TimelineNativeVisualEntity,
            TimelineNativeRelationshipVisual {
                relationship_id: relationship.relationship_id,
                from_node_id: relationship.from_node_id,
                to_node_id: relationship.to_node_id,
                relationship_type: relationship.relationship_type.clone(),
                start_px,
                end_px,
                length_px,
                color_rgb,
            },
            Sprite::from_color(
                Color::srgb(color_rgb[0], color_rgb[1], color_rgb[2]),
                Vec2::new(length_px, TIMELINE_NATIVE_RELATIONSHIP_WIDTH_PX),
            ),
            Transform {
                translation: Vec3::new(center_x, center_y, 0.5),
                rotation: Quat::from_rotation_z(angle),
                ..Default::default()
            },
        ));
    }
}

fn relationship_endpoint_px(
    projection: &TimelineRenderProjection,
    sorted_tracks: &[TimelineRenderTrack],
    layout: TimelineNativeRenderLayout,
    viewport: TimelineViewport,
    node_id: NodeId,
) -> Option<[f32; 2]> {
    let clip = projection
        .clips
        .iter()
        .find(|clip| clip.node_id == node_id)?;
    let center_ms = clip
        .start_ms
        .saturating_add(clip.end_ms.saturating_sub(clip.start_ms) / 2);
    if center_ms < viewport.start_ms || center_ms > viewport.end_ms {
        return None;
    }
    let track_index = sorted_tracks
        .iter()
        .position(|track| track.track_id == clip.track_id)?;
    let ratio = center_ms.saturating_sub(viewport.start_ms) as f32 / viewport.width_ms() as f32;
    let x_px = layout.left_px() + (ratio * layout.usable_width_px());
    let y_px =
        layout.top_px() - (track_index as f32 * (layout.clip_height_px + layout.track_gap_px));
    Some([x_px, y_px])
}

fn spawn_timeline_native_playhead_visual(
    world: &mut World,
    layout: TimelineNativeRenderLayout,
    viewport: TimelineViewport,
    playhead: TimelinePlayhead,
) {
    if playhead.position_ms < viewport.start_ms || playhead.position_ms > viewport.end_ms {
        return;
    }

    let viewport_width_ms = viewport.width_ms();
    let position_ratio =
        playhead.position_ms.saturating_sub(viewport.start_ms) as f32 / viewport_width_ms as f32;
    let x_px = layout.left_px() + (position_ratio * layout.usable_width_px());
    let height_px = (layout.height_px - (layout.top_padding_px * 2.0)).max(layout.clip_height_px);
    let y_px = layout.top_px() - (height_px / 2.0) + (layout.clip_height_px / 2.0);

    world.spawn((
        TimelineSceneEntity,
        TimelineNativeVisualEntity,
        TimelineNativePlayheadVisual {
            position_ms: playhead.position_ms,
            x_px,
            height_px,
        },
        Sprite::from_color(
            Color::srgb(0.937, 0.267, 0.267),
            Vec2::new(TIMELINE_NATIVE_PLAYHEAD_WIDTH_PX, height_px),
        ),
        Transform::from_translation(Vec3::new(x_px, y_px, 1.0)),
    ));
}

fn despawn_existing_timeline_native_visuals(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, bevy::prelude::With<TimelineNativeVisualEntity>>()
        .iter(world)
        .collect();

    for entity in entities {
        let _ = world.despawn(entity);
    }
}
