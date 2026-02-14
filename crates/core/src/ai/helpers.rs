use crate::ai::backend::SurroundingContext;
use crate::timeline::clip::{BeatClip, ClipId};
use crate::timeline::track::ArcTrack;

/// Default context window: number of clips before and after to include.
const CONTEXT_WINDOW: usize = 2;

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
