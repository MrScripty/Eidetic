use eidetic_core::contracts::{TimelineRenderClip, TimelineRenderProjection, TimelineRenderTrack};
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;

mod app;
mod app_command;
#[cfg(feature = "native_render")]
mod native_command;
#[cfg(feature = "native_render")]
mod native_lifecycle;
#[cfg(feature = "native_render")]
mod native_navigation;
#[cfg(feature = "native_render")]
mod native_visual;
mod split;

#[test]
fn crate_keeps_bevy_features_leaf_and_minimal() {
    let manifest = include_str!("../Cargo.toml");

    assert!(manifest.contains("bevy = { version = \"0.18.1\""));
    assert!(manifest.contains("default-features = false"));
    assert!(manifest.contains("features = [\"std\"]"));
    assert!(manifest.contains("[features]"));
    assert!(manifest.contains("default = []"));
    assert!(manifest.contains("native_render = ["));
    assert!(manifest.contains("\"bevy/2d_bevy_render\""));
    assert!(manifest.contains("\"bevy/bevy_winit\""));
    assert!(manifest.contains("\"bevy/bevy_window\""));
    assert!(manifest.contains("\"bevy/wayland\""));
    assert!(manifest.contains("\"bevy/x11\""));
    assert!(!manifest.contains("\"bevy_text\""));
    assert!(!manifest.contains("\"bevy_ui\""));
}

pub(super) fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
    let track_id = TrackId::new();
    projection_with_clip(node_id, track_id, 1_000, 4_000)
}

fn projection_with_clip(
    node_id: NodeId,
    track_id: TrackId,
    start_ms: u64,
    end_ms: u64,
) -> TimelineRenderProjection {
    TimelineRenderProjection {
        total_duration_ms: 10_000,
        selected_node_id: None,
        structure_segments: Vec::new(),
        tracks: vec![TimelineRenderTrack {
            track_id,
            level: StoryLevel::Scene,
            label: "Scenes".to_string(),
            sort_order: 30,
            collapsed: false,
        }],
        clips: vec![TimelineRenderClip {
            node_id,
            parent_id: None,
            track_id,
            level: StoryLevel::Scene,
            name: "Beach argument".to_string(),
            start_ms,
            end_ms,
            sort_order: 10,
            locked: false,
            content_status: ContentStatus::NotesOnly,
            beat_type: None,
            arc_ids: Vec::new(),
        }],
        relationships: Vec::new(),
        gaps: Vec::new(),
        affect_overlays: Vec::new(),
    }
}

fn projection_with_two_tracks(
    upper_node_id: NodeId,
    upper_track_id: TrackId,
    lower_node_id: NodeId,
    lower_track_id: TrackId,
) -> TimelineRenderProjection {
    TimelineRenderProjection {
        total_duration_ms: 10_000,
        selected_node_id: None,
        structure_segments: Vec::new(),
        tracks: vec![
            TimelineRenderTrack {
                track_id: lower_track_id,
                level: StoryLevel::Beat,
                label: "Beats".to_string(),
                sort_order: 20,
                collapsed: false,
            },
            TimelineRenderTrack {
                track_id: upper_track_id,
                level: StoryLevel::Scene,
                label: "Scenes".to_string(),
                sort_order: 10,
                collapsed: false,
            },
        ],
        clips: vec![
            TimelineRenderClip {
                node_id: upper_node_id,
                parent_id: None,
                track_id: upper_track_id,
                level: StoryLevel::Scene,
                name: "Opening scene".to_string(),
                start_ms: 1_000,
                end_ms: 4_000,
                sort_order: 10,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            },
            TimelineRenderClip {
                node_id: lower_node_id,
                parent_id: Some(upper_node_id),
                track_id: lower_track_id,
                level: StoryLevel::Beat,
                name: "Reversal".to_string(),
                start_ms: 2_000,
                end_ms: 5_000,
                sort_order: 20,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            },
        ],
        relationships: Vec::new(),
        gaps: Vec::new(),
        affect_overlays: Vec::new(),
    }
}
