use serde::{Deserialize, Serialize};

use crate::timeline::Timeline;
use crate::timeline::node::{BeatType, ContentStatus, NodeId, StoryLevel, StoryNode};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedNodeEditorProjection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<SelectedNodeEditorNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_level: Option<StoryLevel>,
    #[serde(default)]
    pub has_children: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<SelectedNodeEditorSummary>,
    #[serde(default)]
    pub siblings: Vec<SelectedNodeEditorSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_sibling_index: Option<u32>,
    #[serde(default)]
    pub children: Vec<SelectedNodeEditorSummary>,
    pub adjacent_parents: SelectedNodeEditorAdjacentParents,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedNodeEditorNode {
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    pub level: StoryLevel,
    pub sort_order: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub name: String,
    #[serde(default)]
    pub notes: String,
    pub content_status: ContentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<BeatType>,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedNodeEditorSummary {
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    pub level: StoryLevel,
    pub sort_order: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub name: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<BeatType>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelectedNodeEditorAdjacentParents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<SelectedNodeEditorSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<SelectedNodeEditorSummary>,
}

impl SelectedNodeEditorProjection {
    pub fn empty() -> Self {
        Self {
            node: None,
            child_level: None,
            has_children: false,
            parent: None,
            siblings: Vec::new(),
            current_sibling_index: None,
            children: Vec::new(),
            adjacent_parents: SelectedNodeEditorAdjacentParents::default(),
        }
    }

    pub fn from_timeline(timeline: &Timeline, selected_node_id: Option<NodeId>) -> Option<Self> {
        let Some(selected_node_id) = selected_node_id else {
            return Some(Self::empty());
        };
        let node = timeline
            .nodes
            .iter()
            .find(|node| node.id == selected_node_id)?;
        let parent = node
            .parent_id
            .and_then(|parent_id| timeline.nodes.iter().find(|node| node.id == parent_id))
            .map(SelectedNodeEditorSummary::from);
        let mut siblings = timeline
            .nodes
            .iter()
            .filter(|sibling| sibling.level == node.level && sibling.parent_id == node.parent_id)
            .map(SelectedNodeEditorSummary::from)
            .collect::<Vec<_>>();
        sort_summaries(&mut siblings);
        let current_sibling_index = siblings
            .iter()
            .position(|sibling| sibling.node_id == node.id)
            .map(|index| index as u32);
        let mut children = timeline
            .nodes
            .iter()
            .filter(|child| child.parent_id == Some(node.id))
            .map(SelectedNodeEditorSummary::from)
            .collect::<Vec<_>>();
        sort_summaries(&mut children);

        Some(Self {
            node: Some(SelectedNodeEditorNode::from(node)),
            child_level: node.level.child_level(),
            has_children: !children.is_empty(),
            parent,
            siblings,
            current_sibling_index,
            children,
            adjacent_parents: adjacent_parents_for(timeline, node),
        })
    }
}

impl From<&StoryNode> for SelectedNodeEditorNode {
    fn from(node: &StoryNode) -> Self {
        Self {
            node_id: node.id,
            parent_id: node.parent_id,
            level: node.level,
            sort_order: node.sort_order,
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
            name: node.name.clone(),
            notes: node.content.notes.clone(),
            content_status: node.content.status,
            beat_type: node.beat_type.clone(),
            locked: node.locked,
        }
    }
}

impl From<&StoryNode> for SelectedNodeEditorSummary {
    fn from(node: &StoryNode) -> Self {
        Self {
            node_id: node.id,
            parent_id: node.parent_id,
            level: node.level,
            sort_order: node.sort_order,
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
            name: node.name.clone(),
            notes: node.content.notes.clone(),
            beat_type: node.beat_type.clone(),
        }
    }
}

fn adjacent_parents_for(
    timeline: &Timeline,
    node: &StoryNode,
) -> SelectedNodeEditorAdjacentParents {
    let Some(parent_id) = node.parent_id else {
        return SelectedNodeEditorAdjacentParents::default();
    };
    let Some(parent) = timeline.nodes.iter().find(|node| node.id == parent_id) else {
        return SelectedNodeEditorAdjacentParents::default();
    };
    let grandparent_id = parent.parent_id;
    let mut parents = timeline
        .nodes
        .iter()
        .filter(|candidate| {
            candidate.level == parent.level && candidate.parent_id == grandparent_id
        })
        .collect::<Vec<_>>();
    parents.sort_by_key(|parent| (parent.time_range.start_ms, parent.sort_order, parent.id.0));
    let Some(parent_index) = parents
        .iter()
        .position(|candidate| candidate.id == parent.id)
    else {
        return SelectedNodeEditorAdjacentParents::default();
    };

    SelectedNodeEditorAdjacentParents {
        before: parent_index
            .checked_sub(1)
            .and_then(|index| parents.get(index))
            .map(|node| SelectedNodeEditorSummary::from(*node)),
        after: parents
            .get(parent_index + 1)
            .map(|node| SelectedNodeEditorSummary::from(*node)),
    }
}

fn sort_summaries(summaries: &mut [SelectedNodeEditorSummary]) {
    summaries.sort_by_key(|summary| (summary.start_ms, summary.sort_order, summary.node_id.0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::node::{NodeContent, StoryLevel, StoryNode};
    use crate::timeline::structure::EpisodeStructure;
    use crate::timeline::timing::TimeRange;

    #[test]
    fn selected_node_editor_projection_maps_editor_context() {
        let mut timeline = Timeline::new(100_000, EpisodeStructure::standard_30_min());
        let mut act_one = StoryNode::new(
            "Act one",
            StoryLevel::Act,
            TimeRange::new(0, 30_000).unwrap(),
        );
        act_one.sort_order = 1;
        let act_one_id = act_one.id;
        let mut act_two = StoryNode::new(
            "Act two",
            StoryLevel::Act,
            TimeRange::new(30_000, 60_000).unwrap(),
        );
        act_two.sort_order = 2;
        let sequence = StoryNode {
            parent_id: Some(act_one_id),
            content: NodeContent {
                notes: "Sequence notes".to_string(),
                status: ContentStatus::NotesOnly,
                ..NodeContent::default()
            },
            ..StoryNode::new(
                "Beach sequence",
                StoryLevel::Sequence,
                TimeRange::new(1_000, 10_000).unwrap(),
            )
        };
        let sequence_id = sequence.id;
        let scene = StoryNode {
            parent_id: Some(sequence_id),
            ..StoryNode::new(
                "Rain scene",
                StoryLevel::Scene,
                TimeRange::new(2_000, 3_000).unwrap(),
            )
        };
        timeline.nodes.extend([act_one, act_two, sequence, scene]);

        let projection = SelectedNodeEditorProjection::from_timeline(&timeline, Some(sequence_id))
            .expect("projection");

        assert_eq!(
            projection.node.as_ref().expect("node").name,
            "Beach sequence"
        );
        assert_eq!(
            projection.node.as_ref().expect("node").notes,
            "Sequence notes"
        );
        assert_eq!(
            projection.node.as_ref().expect("node").content_status,
            ContentStatus::NotesOnly
        );
        assert_eq!(projection.child_level, Some(StoryLevel::Scene));
        assert!(projection.has_children);
        assert_eq!(projection.parent.as_ref().expect("parent").name, "Act one");
        assert_eq!(projection.siblings.len(), 1);
        assert_eq!(projection.current_sibling_index, Some(0));
        assert_eq!(projection.children[0].name, "Rain scene");
        assert!(projection.adjacent_parents.before.is_none());
        assert_eq!(
            projection
                .adjacent_parents
                .after
                .as_ref()
                .expect("after")
                .name,
            "Act two"
        );
    }
}
