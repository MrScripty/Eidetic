use crate::timeline::node::{NodeId, StoryNode};
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
    /// The target node's own notes — always included.
    Target = 0,
    /// Parent/ancestor context.
    Ancestor = 1,
    /// Adjacent sibling nodes at the same level.
    Adjacent = 2,
    /// Story bible entities directly referenced by this node (full text).
    BibleReferenced = 3,
    /// Arc descriptions.
    EntityDescriptions = 4,
    /// Story bible entities not directly referenced (compact text).
    BibleNearby = 5,
    /// Content from farther nodes for consistency.
    FartherContext = 6,
}

/// Rough token estimation: ~4 characters per token.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Find nodes at the same level whose time ranges overlap with the target node.
pub fn overlapping_nodes<'a>(
    timeline: &'a Timeline,
    target_node_id: NodeId,
) -> Vec<&'a StoryNode> {
    let Ok(target) = timeline.node(target_node_id) else {
        return vec![];
    };
    let level = target.level;
    let range = target.time_range;

    timeline
        .nodes
        .iter()
        .filter(|n| {
            n.id != target_node_id
                && n.level == level
                && n.time_range.start_ms < range.end_ms
                && n.time_range.end_ms > range.start_ms
        })
        .collect()
}

/// Pack context items into a token budget, returning indices that fit,
/// ordered by priority (highest priority first).
pub fn pack_context(items: &mut [ContextItem], budget_tokens: usize) -> Vec<usize> {
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

    #[test]
    fn estimate_tokens_rough() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hello"), 2); // 5 chars -> (5+3)/4 = 2
        assert_eq!(estimate_tokens("a".repeat(100).as_str()), 25);
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

        let selected = pack_context(&mut items, 30);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&0)); // Target
        assert!(selected.contains(&2)); // Adjacent
        assert!(!selected.contains(&1)); // FartherContext excluded
    }
}
