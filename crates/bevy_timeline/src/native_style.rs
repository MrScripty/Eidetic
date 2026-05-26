use eidetic_core::contracts::{EmotionalIntensity, Valence};
use eidetic_core::timeline::node::{ContentStatus, StoryLevel};
use eidetic_core::timeline::relationship::RelationshipType;

const TIMELINE_NATIVE_AFFECT_MIN_HEIGHT_PX: f32 = 4.0;
const TIMELINE_NATIVE_AFFECT_MAX_HEIGHT_PX: f32 = 10.0;

pub(crate) fn native_clip_color_rgb(
    level: StoryLevel,
    locked: bool,
    content_status: ContentStatus,
) -> [f32; 3] {
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

pub(crate) fn native_relationship_color_rgb(relationship_type: &RelationshipType) -> [f32; 3] {
    match relationship_type {
        RelationshipType::Causal => [0.937, 0.384, 0.314],
        RelationshipType::Convergence { .. } => [0.655, 0.463, 0.914],
        RelationshipType::Thematic => [0.933, 0.831, 0.455],
    }
}

pub(crate) fn native_affect_color_rgb(valence: Valence) -> [f32; 3] {
    match valence.basis_points() {
        value if value < -150 => [0.376, 0.592, 0.827],
        value if value > 150 => [0.282, 0.686, 0.424],
        _ => [0.933, 0.831, 0.455],
    }
}

pub(crate) fn native_affect_height_px(intensity: EmotionalIntensity) -> f32 {
    let ratio = intensity.basis_points() as f32 / 1_000.0;
    TIMELINE_NATIVE_AFFECT_MIN_HEIGHT_PX
        + ((TIMELINE_NATIVE_AFFECT_MAX_HEIGHT_PX - TIMELINE_NATIVE_AFFECT_MIN_HEIGHT_PX) * ratio)
}
