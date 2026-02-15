use crate::ai::backend::{GenerateRequest, PlanBeatsRequest};
use crate::ai::context::overlapping_beats;
use crate::error::{Error, Result};
use crate::project::Project;
use crate::story::bible::{gather_bible_context, BibleContext, ResolvedEntity};
use crate::timeline::clip::ClipId;

use super::helpers::{gather_recap_context, gather_surrounding_scripts, gather_surrounding_sub_beat_scripts};

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

    // For sub-beats, gather surrounding context from sibling sub-beats.
    // For main clips, gather from the main track as before.
    let mut surrounding_context = if beat_clip.parent_clip_id.is_some() {
        gather_surrounding_sub_beat_scripts(track, clip_id)
    } else {
        gather_surrounding_scripts(track, clip_id)
    };

    // Gather cross-track recaps for continuity.
    surrounding_context.preceding_recaps =
        gather_recap_context(&project.timeline, &project.arcs, clip_id);

    // Gather bible context resolved at the beat's midpoint time.
    let beat_mid_ms = beat_clip.time_range.start_ms
        + beat_clip.time_range.duration_ms() / 2;
    let bible_context = gather_bible_context(&project.bible, clip_id, beat_mid_ms);

    let time_budget_ms = beat_clip.time_range.duration_ms();

    // If this is a sub-beat, gather parent context and sibling outlines.
    let (parent_scene_notes, sibling_beat_outlines) =
        if let Some(parent_id) = beat_clip.parent_clip_id {
            let parent_notes = timeline
                .clip(parent_id)
                .ok()
                .map(|p| p.content.beat_notes.clone());

            let siblings: Vec<(String, String, String)> = track
                .sub_beats_for_clip(parent_id)
                .into_iter()
                .map(|b| {
                    (
                        b.name.clone(),
                        format!("{:?}", b.beat_type),
                        b.content.beat_notes.clone(),
                    )
                })
                .collect();

            (parent_notes, siblings)
        } else {
            (None, vec![])
        };

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
        parent_scene_notes,
        sibling_beat_outlines,
    })
}

/// Build a [`PlanBeatsRequest`] for decomposing a scene clip into beats.
///
/// Gathers the same context as `build_generate_request` minus overlapping beats
/// and RAG, since beat planning only needs the scene's own context.
pub fn build_plan_beats_request(project: &Project, clip_id: ClipId) -> Result<PlanBeatsRequest> {
    let timeline = &project.timeline;

    let beat_clip = timeline.clip(clip_id)?.clone();

    let track = timeline
        .track_for_clip(clip_id)
        .ok_or(Error::ClipNotFound(clip_id.0))?;

    let arc = project
        .arcs
        .iter()
        .find(|a| a.id == track.arc_id)
        .ok_or(Error::ArcNotFound(track.arc_id.0))?
        .clone();

    let mut surrounding_context = gather_surrounding_scripts(track, clip_id);
    surrounding_context.preceding_recaps =
        gather_recap_context(&project.timeline, &project.arcs, clip_id);

    let beat_mid_ms =
        beat_clip.time_range.start_ms + beat_clip.time_range.duration_ms() / 2;

    // For beat planning, include ALL entities with full detail.
    // Unlike generation, the clip hasn't been processed yet so clip_refs
    // won't reference it — gather_bible_context would put everything in
    // "nearby" (compact only). The planner needs full entity info to
    // reference characters, locations, and props in beat outlines.
    let bible_context = BibleContext {
        referenced_entities: project
            .bible
            .entities
            .iter()
            .map(|e| ResolvedEntity {
                entity_id: e.id,
                name: e.name.clone(),
                category: e.category.clone(),
                compact_text: e.to_prompt_text(beat_mid_ms),
                full_text: Some(e.to_full_prompt_text(beat_mid_ms)),
            })
            .collect(),
        nearby_entities: Vec::new(),
    };

    Ok(PlanBeatsRequest {
        beat_clip,
        arc,
        bible_context,
        surrounding_context,
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
