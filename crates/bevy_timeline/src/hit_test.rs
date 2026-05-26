use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::track::TrackId;

use crate::{
    TimelineRendererError, TimelineViewport, TimelineViewportGeometry, TimelineViewportPoint,
};

pub fn hit_test_clip_at_time(
    projection: &TimelineRenderProjection,
    track_id: TrackId,
    time_ms: u64,
) -> Option<NodeId> {
    projection
        .clips
        .iter()
        .filter(|clip| {
            clip.track_id == track_id && clip.start_ms <= time_ms && time_ms < clip.end_ms
        })
        .max_by_key(|clip| (clip.sort_order, clip.start_ms))
        .map(|clip| clip.node_id)
}

pub fn hit_test_clip_at_point(
    projection: &TimelineRenderProjection,
    viewport: TimelineViewport,
    geometry: TimelineViewportGeometry,
    point: TimelineViewportPoint,
) -> Result<Option<NodeId>, TimelineRendererError> {
    if !geometry.validate() {
        return Err(TimelineRendererError::InvalidViewportGeometry {
            width_px: geometry.width_px,
            height_px: geometry.height_px,
            track_height_px: geometry.track_height_px,
        });
    }
    if point.x_px >= geometry.width_px || point.y_px >= geometry.height_px {
        return Ok(None);
    }

    let track_index = point.y_px / geometry.track_height_px;
    let mut tracks = projection.tracks.iter().collect::<Vec<_>>();
    tracks.sort_by_key(|track| (track.sort_order, track.track_id.0));
    let Some(track) = tracks.get(track_index as usize) else {
        return Ok(None);
    };

    let width_ms = u128::from(viewport.width_ms());
    let offset_ms = u128::from(point.x_px) * width_ms / u128::from(geometry.width_px);
    let time_ms = viewport
        .start_ms
        .saturating_add(u64::try_from(offset_ms).unwrap_or(u64::MAX));

    Ok(hit_test_clip_at_time(projection, track.track_id, time_ms))
}
