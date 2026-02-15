use crate::ai::backend::{RecapEntry, SurroundingContext};
use crate::story::arc::StoryArc;
use crate::timeline::clip::{BeatClip, ClipId};
use crate::timeline::track::ArcTrack;
use crate::timeline::Timeline;

/// Default context window: number of clips before and after to include.
const CONTEXT_WINDOW: usize = 2;

/// Maximum number of recaps to include across all tracks.
const MAX_RECAPS: usize = 6;

/// Gather surrounding scripts from adjacent clips on the same track.
///
/// Looks at up to `CONTEXT_WINDOW` clips before and after the target,
/// collecting any generated or user-refined scripts.
pub fn gather_surrounding_scripts(
    track: &ArcTrack,
    clip_id: ClipId,
) -> SurroundingContext {
    let Some(idx) = track.clips.iter().position(|c| c.id == clip_id) else {
        return SurroundingContext::default();
    };

    let preceding_scripts = track.clips[idx.saturating_sub(CONTEXT_WINDOW)..idx]
        .iter()
        .filter_map(|c| best_script(c))
        .collect();

    let after_start = idx + 1;
    let after_end = (after_start + CONTEXT_WINDOW).min(track.clips.len());
    let following_scripts = track.clips[after_start..after_end]
        .iter()
        .filter_map(|c| best_script(c))
        .collect();

    SurroundingContext {
        preceding_scripts,
        following_scripts,
        preceding_recaps: Vec::new(),
    }
}

/// Gather surrounding scripts from sibling sub-beats on the same track.
///
/// For a sub-beat, the "surrounding" context is the other sub-beats
/// belonging to the same parent clip, ordered by time.
pub fn gather_surrounding_sub_beat_scripts(
    track: &ArcTrack,
    clip_id: ClipId,
) -> SurroundingContext {
    // Find this sub-beat's parent.
    let parent_id = match track.sub_beats.iter().find(|b| b.id == clip_id) {
        Some(b) => match b.parent_clip_id {
            Some(pid) => pid,
            None => return SurroundingContext::default(),
        },
        None => return SurroundingContext::default(),
    };

    // Get all siblings sorted by time.
    let mut siblings: Vec<&BeatClip> = track.sub_beats_for_clip(parent_id);
    siblings.sort_by_key(|b| b.time_range.start_ms);

    let Some(idx) = siblings.iter().position(|b| b.id == clip_id) else {
        return SurroundingContext::default();
    };

    let preceding_scripts = siblings[idx.saturating_sub(CONTEXT_WINDOW)..idx]
        .iter()
        .filter_map(|c| best_script_or_outline(c))
        .collect();

    let after_start = idx + 1;
    let after_end = (after_start + CONTEXT_WINDOW).min(siblings.len());
    let following_scripts = siblings[after_start..after_end]
        .iter()
        .filter_map(|c| best_script_or_outline(c))
        .collect();

    SurroundingContext {
        preceding_scripts,
        following_scripts,
        preceding_recaps: Vec::new(),
    }
}

/// Return the best available script text for a clip (user-refined > generated).
pub fn best_script(clip: &BeatClip) -> Option<String> {
    clip.content
        .user_refined_script
        .as_ref()
        .or(clip.content.generated_script.as_ref())
        .cloned()
}

/// Return the best available context for a sub-beat:
/// script if available, otherwise beat notes (outline from planning).
pub fn best_script_or_outline(clip: &BeatClip) -> Option<String> {
    best_script(clip).or_else(|| {
        let notes = &clip.content.beat_notes;
        if notes.trim().is_empty() {
            None
        } else {
            Some(format!("[BEAT OUTLINE: {} ({:?})]\n{}", clip.name, clip.beat_type, notes))
        }
    })
}

/// Gather scene recaps from ALL tracks for clips that end before the
/// target clip's start time. Returns the most recent recaps, ordered
/// chronologically (earliest first).
///
/// This provides cross-track continuity: when generating for Track A Clip 3,
/// we include recaps from Track B Clip 1 and Track B Clip 2 if they
/// temporally precede it.
pub fn gather_recap_context(
    timeline: &Timeline,
    arcs: &[StoryArc],
    target_clip_id: ClipId,
) -> Vec<RecapEntry> {
    let Ok(target_clip) = timeline.clip(target_clip_id) else {
        return vec![];
    };
    let target_start = target_clip.time_range.start_ms;

    let mut entries: Vec<RecapEntry> = Vec::new();

    for track in &timeline.tracks {
        let arc_name = arcs
            .iter()
            .find(|a| a.id == track.arc_id)
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown Arc");

        for clip in &track.clips {
            if clip.id == target_clip_id {
                continue;
            }
            // Only include clips that end before or at the target's start.
            if clip.time_range.end_ms > target_start {
                continue;
            }
            if let Some(ref recap) = clip.content.scene_recap {
                entries.push(RecapEntry {
                    arc_name: arc_name.to_string(),
                    clip_name: clip.name.clone(),
                    end_time_ms: clip.time_range.end_ms,
                    recap: recap.clone(),
                });
            }
        }
    }

    // Sort chronologically by end time (ascending).
    entries.sort_by_key(|e| e.end_time_ms);

    // Keep only the most recent MAX_RECAPS.
    if entries.len() > MAX_RECAPS {
        entries = entries.split_off(entries.len() - MAX_RECAPS);
    }

    entries
}
