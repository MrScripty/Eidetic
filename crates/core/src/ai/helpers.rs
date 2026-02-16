use crate::ai::backend::{RecapEntry, SurroundingContext};
use crate::story::arc::StoryArc;
use crate::timeline::node::{NodeId, StoryLevel, StoryNode};
use crate::timeline::Timeline;

/// Default context window: number of sibling nodes before and after to include.
const CONTEXT_WINDOW: usize = 2;

/// Maximum number of recaps to include.
const MAX_RECAPS: usize = 6;

/// Gather surrounding content from sibling nodes (same parent, same level).
///
/// Looks at up to `CONTEXT_WINDOW` siblings before and after the target,
/// collecting any generated or user-refined content.
pub fn gather_surrounding_context(
    timeline: &Timeline,
    node_id: NodeId,
) -> SurroundingContext {
    let Ok(node) = timeline.node(node_id) else {
        return SurroundingContext::default();
    };

    let siblings = timeline.siblings_of(node_id);

    // Find where the target node would sit chronologically among siblings.
    let target_start = node.time_range.start_ms;

    // Split siblings into before and after by time.
    let preceding: Vec<&StoryNode> = siblings
        .iter()
        .filter(|s| s.time_range.start_ms < target_start)
        .copied()
        .collect();

    let following: Vec<&StoryNode> = siblings
        .iter()
        .filter(|s| s.time_range.start_ms > target_start)
        .copied()
        .collect();

    // Take the last CONTEXT_WINDOW preceding and first CONTEXT_WINDOW following.
    let preceding_scripts = preceding
        .iter()
        .rev()
        .take(CONTEXT_WINDOW)
        .rev()
        .filter_map(|n| best_text(n))
        .collect();

    let following_scripts = following
        .iter()
        .take(CONTEXT_WINDOW)
        .filter_map(|n| best_text(n))
        .collect();

    SurroundingContext {
        preceding_scripts,
        following_scripts,
        preceding_recaps: Vec::new(),
    }
}

/// Return the script/outline content for a node, if any.
pub fn best_text(node: &StoryNode) -> Option<String> {
    if node.content.content.is_empty() {
        None
    } else {
        Some(node.content.content.clone())
    }
}

/// Return the best available context for a node:
/// text if available, otherwise notes (outline from planning).
pub fn best_text_or_outline(node: &StoryNode) -> Option<String> {
    best_text(node).or_else(|| {
        let notes = &node.content.notes;
        if notes.trim().is_empty() {
            None
        } else {
            let type_label = node
                .beat_type
                .as_ref()
                .map(|bt| format!("{:?}", bt))
                .unwrap_or_else(|| node.level.to_string());
            Some(format!("[OUTLINE: {} ({})]\n{}", node.name, type_label, notes))
        }
    })
}

/// Gather scene recaps from preceding nodes for continuity context.
///
/// Looks at Scene-level nodes that end before the target node's start time
/// and have scene recaps. For Beat-level targets, looks at sibling beats
/// and preceding scenes.
pub fn gather_recap_context(
    timeline: &Timeline,
    arcs: &[StoryArc],
    target_node_id: NodeId,
) -> Vec<RecapEntry> {
    let Ok(target) = timeline.node(target_node_id) else {
        return vec![];
    };
    let target_start = target.time_range.start_ms;

    let mut entries: Vec<RecapEntry> = Vec::new();

    // Gather recaps from all Scene-level nodes that precede the target.
    for node in &timeline.nodes {
        if node.id == target_node_id {
            continue;
        }
        // Only include nodes that end before or at the target's start.
        if node.time_range.end_ms > target_start {
            continue;
        }
        // Only include Scene and Beat level nodes (they have recaps).
        if node.level != StoryLevel::Scene && node.level != StoryLevel::Beat {
            continue;
        }
        if let Some(ref recap) = node.content.scene_recap {
            // Find arc names for this node.
            let arc_ids = timeline.arcs_for_node(node.id);
            let arc_name = arc_ids
                .first()
                .and_then(|aid| arcs.iter().find(|a| a.id == *aid))
                .map(|a| a.name.as_str())
                .unwrap_or("Untagged");

            entries.push(RecapEntry {
                arc_name: arc_name.to_string(),
                node_name: node.name.clone(),
                end_time_ms: node.time_range.end_ms,
                recap: recap.clone(),
            });
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
