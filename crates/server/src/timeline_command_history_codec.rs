use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{BeatType, ContentStatus, StoryLevel};
use eidetic_core::timeline::relationship::RelationshipType;

use crate::timeline_command::TimelineCommandError;

pub(crate) fn encode_content_status(status: ContentStatus) -> String {
    match status {
        ContentStatus::Empty => "Empty",
        ContentStatus::NotesOnly => "NotesOnly",
        ContentStatus::Generating => "Generating",
        ContentStatus::HasContent => "HasContent",
    }
    .to_string()
}

pub(crate) fn encode_story_level(level: StoryLevel) -> String {
    level.label().to_string()
}

pub(crate) fn encode_beat_type(beat_type: &BeatType) -> Result<String, TimelineCommandError> {
    serde_json::to_string(beat_type).map_err(|error| {
        TimelineCommandError::Core(eidetic_core::Error::InvalidOperation(format!(
            "invalid beat type: {error}"
        )))
    })
}

pub(crate) fn encode_relationship_type(
    relationship_type: &RelationshipType,
) -> Result<String, TimelineCommandError> {
    serde_json::to_string(relationship_type).map_err(|error| {
        TimelineCommandError::Core(eidetic_core::Error::InvalidOperation(format!(
            "invalid relationship type: {error}"
        )))
    })
}

pub(crate) fn encode_arc_ids(arc_ids: &[ArcId]) -> Result<String, TimelineCommandError> {
    let values: Vec<String> = arc_ids.iter().map(|arc_id| arc_id.0.to_string()).collect();
    serde_json::to_string(&values).map_err(|error| {
        TimelineCommandError::Core(eidetic_core::Error::InvalidOperation(format!(
            "invalid arc ids: {error}"
        )))
    })
}
