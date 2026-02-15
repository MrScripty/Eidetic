use crate::ai::backend::GenerateRequest;
use crate::ai::context::overlapping_beats;
use crate::error::{Error, Result};
use crate::project::Project;
use crate::story::bible::gather_bible_context;
use crate::timeline::clip::ClipId;

use super::helpers::{gather_recap_context, gather_surrounding_scripts};

/// Build a [`GenerateRequest`] for a specific beat clip from the project state.
///
/// Gathers:
/// - The target clip and its parent arc
/// - Overlapping beats from other tracks at the same time position
/// - Story bible entities resolved at this beat's time position
/// - Surrounding scripts (up to 2 preceding and 2 following) from the same arc track
pub fn build_generate_request(project: &Project, clip_id: ClipId) -> Result<GenerateRequest> {
    let timeline = &project.timeline;

    // Find the target clip.
    let beat_clip = timeline.clip(clip_id)?.clone();

    // Find which track (and therefore which arc) owns this clip.
    let track = timeline
        .track_for_clip(clip_id)
        .ok_or(Error::ClipNotFound(clip_id.0))?;

    let arc = project
        .arcs
        .iter()
        .find(|a| a.id == track.arc_id)
        .ok_or(Error::ArcNotFound(track.arc_id.0))?
        .clone();

    // Collect overlapping beats from other arc tracks.
    let overlapping = overlapping_beats(timeline, clip_id)
        .into_iter()
        .filter_map(|(ov_track, ov_clip)| {
            let ov_arc = project.arcs.iter().find(|a| a.id == ov_track.arc_id)?;
            Some((ov_clip.clone(), ov_arc.clone()))
        })
        .collect();

    // Gather surrounding scripts from the same track (up to 2 before, 2 after).
    let mut surrounding_context = gather_surrounding_scripts(track, clip_id);

    // Gather cross-track recaps for continuity.
    surrounding_context.preceding_recaps =
        gather_recap_context(&project.timeline, &project.arcs, clip_id);

    // Gather bible context resolved at the beat's midpoint time.
    let beat_mid_ms = beat_clip.time_range.start_ms
        + beat_clip.time_range.duration_ms() / 2;
    let bible_context = gather_bible_context(&project.bible, clip_id, beat_mid_ms);

    let time_budget_ms = beat_clip.time_range.duration_ms();

    Ok(GenerateRequest {
        beat_clip,
        arc,
        overlapping_beats: overlapping,
        bible_context,
        surrounding_context,
        time_budget_ms,
        user_written_anchors: vec![],
        style_notes: None,
        rag_context: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Template;

    #[test]
    fn build_request_from_template() {
        let project = Template::MultiCam.build_project("Test");
        let timeline = &project.timeline;

        let first_clip_id = timeline.tracks[0].clips[0].id;
        let req = build_generate_request(&project, first_clip_id).unwrap();

        assert_eq!(req.beat_clip.id, first_clip_id);
        assert_eq!(req.arc.id, timeline.tracks[0].arc_id);
        assert_eq!(req.time_budget_ms, req.beat_clip.time_range.duration_ms());
    }

    #[test]
    fn surrounding_scripts_window() {
        let project = Template::MultiCam.build_project("Test");
        let timeline = &project.timeline;
        let track = &timeline.tracks[0];

        // First clip has no preceding scripts.
        let ctx = gather_surrounding_scripts(track, track.clips[0].id);
        assert!(ctx.preceding_scripts.is_empty());

        // Last clip has no following scripts.
        let last = track.clips.last().unwrap();
        let ctx = gather_surrounding_scripts(track, last.id);
        assert!(ctx.following_scripts.is_empty());
    }

    #[test]
    fn build_request_clip_not_found() {
        let project = Template::MultiCam.build_project("Test");
        let bogus_id = ClipId::new();
        let result = build_generate_request(&project, bogus_id);
        assert!(result.is_err());
    }
}
