use crate::ai::backend::{EditContext, SurroundingContext};
use crate::error::{Error, Result};
use crate::project::Project;
use crate::timeline::clip::ClipId;
use crate::timeline::relationship::RelationshipType;

/// Build an [`EditContext`] for the consistency reaction pipeline.
///
/// Compares the clip's `generated_script` (the "before") with its
/// `user_refined_script` (the "after"), plus surrounding scripts
/// for context.
pub fn build_edit_context(project: &Project, clip_id: ClipId) -> Result<EditContext> {
    let clip = project.timeline.clip(clip_id)?.clone();

    let previous_script = clip
        .content
        .generated_script
        .clone()
        .unwrap_or_default();

    let new_script = clip
        .content
        .user_refined_script
        .clone()
        .unwrap_or_default();

    let track = project
        .timeline
        .track_for_clip(clip_id)
        .ok_or(Error::ClipNotFound(clip_id.0))?;

    let surrounding_context = gather_surrounding(track, clip_id);

    Ok(EditContext {
        beat_clip: clip,
        previous_script,
        new_script,
        surrounding_context,
    })
}

/// Find clip IDs that are downstream of the edited clip and might need updates.
///
/// "Downstream" means:
/// - Clips that come after this one on the same track (chronologically)
/// - Clips connected via Causal relationships from this clip
///
/// Locked clips are excluded (user has taken ownership of their content).
pub fn downstream_clip_ids(project: &Project, clip_id: ClipId) -> Vec<ClipId> {
    let mut ids = Vec::new();

    // Clips after this one on the same track.
    if let Some(track) = project.timeline.track_for_clip(clip_id) {
        if let Some(idx) = track.clips.iter().position(|c| c.id == clip_id) {
            for clip in &track.clips[idx + 1..] {
                if !clip.locked && has_script(clip) {
                    ids.push(clip.id);
                }
            }
        }
    }

    // Clips connected via causal relationships from this clip.
    for rel in &project.timeline.relationships {
        if rel.from_clip == clip_id && matches!(rel.relationship_type, RelationshipType::Causal) {
            if let Ok(target) = project.timeline.clip(rel.to_clip) {
                if !target.locked && has_script(target) && !ids.contains(&rel.to_clip) {
                    ids.push(rel.to_clip);
                }
            }
        }
    }

    ids
}

fn has_script(clip: &crate::timeline::clip::BeatClip) -> bool {
    clip.content.generated_script.is_some() || clip.content.user_refined_script.is_some()
}

fn gather_surrounding(
    track: &crate::timeline::track::ArcTrack,
    clip_id: ClipId,
) -> SurroundingContext {
    let Some(idx) = track.clips.iter().position(|c| c.id == clip_id) else {
        return SurroundingContext::default();
    };

    let preceding_scripts = track.clips[idx.saturating_sub(2)..idx]
        .iter()
        .filter_map(|c| best_script(c))
        .collect();

    let start = idx + 1;
    let end = (start + 2).min(track.clips.len());
    let following_scripts = track.clips[start..end]
        .iter()
        .filter_map(|c| best_script(c))
        .collect();

    SurroundingContext {
        preceding_scripts,
        following_scripts,
    }
}

fn best_script(clip: &crate::timeline::clip::BeatClip) -> Option<String> {
    clip.content
        .user_refined_script
        .as_ref()
        .or(clip.content.generated_script.as_ref())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Template;

    #[test]
    fn downstream_excludes_locked_clips() {
        let mut project = Template::MultiCam.build_project("Test");
        let track = &mut project.timeline.tracks[0];
        // Lock the second clip.
        if track.clips.len() > 1 {
            track.clips[1].locked = true;
        }

        let first_id = project.timeline.tracks[0].clips[0].id;
        let downstream = downstream_clip_ids(&project, first_id);
        // The locked clip should be excluded.
        for id in &downstream {
            let clip = project.timeline.clip(*id).unwrap();
            assert!(!clip.locked);
        }
    }

    #[test]
    fn build_edit_context_returns_scripts() {
        let mut project = Template::MultiCam.build_project("Test");
        let clip_id = project.timeline.tracks[0].clips[0].id;
        // Set some script content.
        let clip = project.timeline.clip_mut(clip_id).unwrap();
        clip.content.generated_script = Some("Original script.".into());
        clip.content.user_refined_script = Some("Edited script.".into());

        let ctx = build_edit_context(&project, clip_id).unwrap();
        assert_eq!(ctx.previous_script, "Original script.");
        assert_eq!(ctx.new_script, "Edited script.");
    }
}
