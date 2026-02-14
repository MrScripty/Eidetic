use crate::timeline::clip::{BeatClip, ClipId};
use crate::timeline::track::ArcTrack;
use crate::timeline::Timeline;

/// A prioritized context item with estimated token count.
#[derive(Debug)]
pub struct ContextItem {
    pub content: String,
    pub priority: ContextPriority,
    pub estimated_tokens: usize,
}

/// Priority levels for context packing. Lower numeric value = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextPriority {
    /// The target beat's own notes — always included.
    Target = 0,
    /// Beats overlapping in time (for scene weaving).
    Overlapping = 1,
    /// Adjacent beats on the same arc track.
    Adjacent = 2,
    /// Arc and character descriptions.
    EntityDescriptions = 3,
    /// Scripts from farther beats for consistency.
    FartherContext = 4,
}

/// Rough token estimation: ~4 characters per token.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Collect beats that overlap with the target clip's time range on other tracks.
pub fn overlapping_beats<'a>(
    timeline: &'a Timeline,
    target_clip_id: ClipId,
) -> Vec<(&'a ArcTrack, &'a BeatClip)> {
    let Ok(target) = timeline.clip(target_clip_id) else {
        return vec![];
    };
    let mid = target.time_range.start_ms + target.time_range.duration_ms() / 2;
    timeline
        .clips_at(mid)
        .into_iter()
        .filter(|(_, clip)| clip.id != target_clip_id)
        .collect()
}

/// Pack context items into a token budget, returning those that fit,
/// ordered by priority (highest priority first).
pub fn pack_context(items: &mut [ContextItem], budget_tokens: usize) -> Vec<usize> {
    // Sort by priority (lowest enum value = highest priority).
    let mut indices: Vec<usize> = (0..items.len()).collect();
    indices.sort_by_key(|&i| items[i].priority);

    let mut used = 0;
    let mut selected = vec![];
    for i in indices {
        if used + items[i].estimated_tokens <= budget_tokens {
            used += items[i].estimated_tokens;
            selected.push(i);
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Template;

    #[test]
    fn estimate_tokens_rough() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hello"), 2); // 5 chars -> (5+3)/4 = 2
        assert_eq!(estimate_tokens("a".repeat(100).as_str()), 25); // 100/4 = 25
    }

    #[test]
    fn pack_context_respects_budget() {
        let mut items = vec![
            ContextItem {
                content: "high priority".into(),
                priority: ContextPriority::Target,
                estimated_tokens: 10,
            },
            ContextItem {
                content: "low priority".into(),
                priority: ContextPriority::FartherContext,
                estimated_tokens: 90,
            },
            ContextItem {
                content: "medium priority".into(),
                priority: ContextPriority::Adjacent,
                estimated_tokens: 15,
            },
        ];

        // Budget of 30: should fit Target (10) + Adjacent (15) = 25, but not FartherContext (90).
        let selected = pack_context(&mut items, 30);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&0)); // Target
        assert!(selected.contains(&2)); // Adjacent
        assert!(!selected.contains(&1)); // FartherContext excluded
    }

    #[test]
    fn overlapping_beats_from_template() {
        let project = Template::MultiCam.build_project("Test");
        let timeline = &project.timeline;

        // B-plot "Setup" (90K–240K, midpoint 165K) overlaps with A-plot "Complication" (150K–360K).
        let b_track = &timeline.tracks[1];
        let b_setup_id = b_track.clips[0].id;

        let overlaps = overlapping_beats(timeline, b_setup_id);
        assert!(
            !overlaps.is_empty(),
            "B-plot Setup midpoint should overlap with A-plot Complication"
        );
    }
}
