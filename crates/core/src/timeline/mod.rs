pub mod node;
pub mod relationship;
pub mod structure;
pub mod timing;
pub mod track;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::story::arc::ArcId;
use node::{NodeArc, NodeId, StoryLevel, StoryNode};
use relationship::{Relationship, RelationshipId};
use structure::EpisodeStructure;
use timing::TimeRange;
use track::{Track, TrackId};

/// A gap on a track where no story node exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineGap {
    pub level: StoryLevel,
    pub time_range: TimeRange,
    pub preceding_node_id: Option<NodeId>,
    pub following_node_id: Option<NodeId>,
}

/// The central data structure: a timeline with hierarchy-level tracks.
///
/// Represents the full runtime of an episode (~22 min for 30-min TV). Tracks
/// are organized by story level (Act, Sequence, Scene, Beat). Nodes at each
/// level form a tree via `parent_id`. Arcs are tagged onto nodes via `node_arcs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Total episode duration (typically ~22 min content for a 30-min slot).
    pub total_duration_ms: u64,
    /// One track per hierarchy level.
    pub tracks: Vec<Track>,
    /// All story nodes, flat list. Tree structure via `parent_id`.
    pub nodes: Vec<StoryNode>,
    /// Many-to-many arc tagging for nodes.
    #[serde(default)]
    pub node_arcs: Vec<NodeArc>,
    /// Edge-bundled curves connecting nodes.
    pub relationships: Vec<Relationship>,
    /// Act structure (cold open, acts, commercial breaks, tag).
    pub structure: EpisodeStructure,
}

impl Timeline {
    /// Create a new empty timeline with the given duration and structure.
    pub fn new(total_duration_ms: u64, structure: EpisodeStructure) -> Self {
        Self {
            total_duration_ms,
            tracks: Track::default_set(),
            nodes: Vec::new(),
            node_arcs: Vec::new(),
            relationships: Vec::new(),
            structure,
        }
    }

    // ────────────────── Track operations ──────────────────

    /// Find a track by ID.
    pub fn track(&self, id: TrackId) -> Result<&Track> {
        self.tracks
            .iter()
            .find(|t| t.id == id)
            .ok_or(Error::TrackNotFound(id.0))
    }

    /// Find a track by ID (mutable).
    pub fn track_mut(&mut self, id: TrackId) -> Result<&mut Track> {
        self.tracks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(Error::TrackNotFound(id.0))
    }

    /// Find a track by story level.
    pub fn track_for_level(&self, level: StoryLevel) -> Option<&Track> {
        self.tracks.iter().find(|t| t.level == level)
    }

    // ────────────────── Node lookups ──────────────────

    /// Find a node by ID.
    pub fn node(&self, id: NodeId) -> Result<&StoryNode> {
        self.nodes
            .iter()
            .find(|n| n.id == id)
            .ok_or(Error::NodeNotFound(id.0))
    }

    /// Find a node by ID (mutable).
    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut StoryNode> {
        self.nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(Error::NodeNotFound(id.0))
    }

    /// Get all nodes at a given hierarchy level, sorted by start time.
    pub fn nodes_at_level(&self, level: StoryLevel) -> Vec<&StoryNode> {
        let mut nodes: Vec<&StoryNode> = self
            .nodes
            .iter()
            .filter(|n| n.level == level)
            .collect();
        nodes.sort_by_key(|n| n.time_range.start_ms);
        nodes
    }

    /// Get direct children of a parent node, sorted by sort_order then start time.
    pub fn children_of(&self, parent_id: NodeId) -> Vec<&StoryNode> {
        let mut children: Vec<&StoryNode> = self
            .nodes
            .iter()
            .filter(|n| n.parent_id == Some(parent_id))
            .collect();
        children.sort_by_key(|n| (n.sort_order, n.time_range.start_ms));
        children
    }

    /// Get all descendants of a node (recursive).
    pub fn descendants_of(&self, parent_id: NodeId) -> Vec<&StoryNode> {
        let mut result = Vec::new();
        let mut stack = vec![parent_id];
        while let Some(pid) = stack.pop() {
            for node in &self.nodes {
                if node.parent_id == Some(pid) {
                    result.push(node);
                    stack.push(node.id);
                }
            }
        }
        result.sort_by_key(|n| (n.level, n.time_range.start_ms));
        result
    }

    /// Walk up from a node to the root, returning ancestors in order
    /// (immediate parent first, root last).
    pub fn ancestors_of(&self, node_id: NodeId) -> Vec<&StoryNode> {
        let mut ancestors = Vec::new();
        let mut current = self.node(node_id).ok();
        while let Some(node) = current {
            if let Some(pid) = node.parent_id {
                if let Ok(parent) = self.node(pid) {
                    ancestors.push(parent);
                    current = Some(parent);
                    continue;
                }
            }
            break;
        }
        ancestors
    }

    /// Get sibling nodes (same parent, same level, excluding self).
    pub fn siblings_of(&self, node_id: NodeId) -> Vec<&StoryNode> {
        let node = match self.node(node_id) {
            Ok(n) => n,
            Err(_) => return Vec::new(),
        };
        let parent_id = node.parent_id;
        let level = node.level;
        let mut siblings: Vec<&StoryNode> = self
            .nodes
            .iter()
            .filter(|n| n.parent_id == parent_id && n.level == level && n.id != node_id)
            .collect();
        siblings.sort_by_key(|n| (n.sort_order, n.time_range.start_ms));
        siblings
    }

    // ────────────────── Node mutations ──────────────────

    /// Add a node to the timeline, validating it fits within bounds.
    pub fn add_node(&mut self, node: StoryNode) -> Result<()> {
        if node.time_range.end_ms > self.total_duration_ms {
            return Err(Error::NodeExceedsTimeline {
                node_end_ms: node.time_range.end_ms,
                timeline_ms: self.total_duration_ms,
            });
        }
        node.time_range.validate()?;

        // Validate parent-child level relationship.
        if let Some(parent_id) = node.parent_id {
            let parent = self.node(parent_id)?;
            let expected_child_level = parent.level.child_level().ok_or_else(|| {
                Error::InvalidHierarchy(format!(
                    "{} nodes cannot have children",
                    parent.level
                ))
            })?;
            if node.level != expected_child_level {
                return Err(Error::InvalidHierarchy(format!(
                    "expected {} child for {} parent, got {}",
                    expected_child_level, parent.level, node.level
                )));
            }
        } else if node.level != StoryLevel::Premise {
            // Only Premise-level nodes can be parentless.
            return Err(Error::InvalidHierarchy(format!(
                "{} nodes must have a parent",
                node.level
            )));
        } else if self.nodes.iter().any(|n| n.level == StoryLevel::Premise) {
            // Only one Premise node is allowed per timeline.
            return Err(Error::InvalidHierarchy(
                "only one Premise node is allowed".to_string(),
            ));
        }

        self.nodes.push(node);
        Ok(())
    }

    /// Remove a node by ID. Cascades to all descendants.
    /// Also removes relationships and arc tags referencing removed nodes.
    pub fn remove_node(&mut self, id: NodeId) -> Result<StoryNode> {
        let idx = self
            .nodes
            .iter()
            .position(|n| n.id == id)
            .ok_or(Error::NodeNotFound(id.0))?;

        let node = self.nodes.remove(idx);

        // Collect all descendant IDs.
        let descendant_ids: Vec<NodeId> = self.descendants_of(id).iter().map(|n| n.id).collect();

        // Remove descendants.
        self.nodes.retain(|n| !descendant_ids.contains(&n.id));

        // Collect all removed IDs (node + descendants).
        let mut all_removed = descendant_ids;
        all_removed.push(id);

        // Clean up relationships.
        self.relationships.retain(|r| {
            !all_removed.contains(&r.from_node) && !all_removed.contains(&r.to_node)
        });

        // Clean up arc tags.
        self.node_arcs
            .retain(|na| !all_removed.contains(&na.node_id));

        Ok(node)
    }

    /// Move/resize a node to a new time range, proportionally adjusting all descendants.
    pub fn resize_node(&mut self, node_id: NodeId, new_range: TimeRange) -> Result<()> {
        new_range.validate()?;
        if new_range.end_ms > self.total_duration_ms {
            return Err(Error::NodeExceedsTimeline {
                node_end_ms: new_range.end_ms,
                timeline_ms: self.total_duration_ms,
            });
        }

        let node = self.node(node_id)?;
        let old_range = node.time_range;
        let old_duration = old_range.end_ms - old_range.start_ms;
        let new_duration = new_range.end_ms - new_range.start_ms;

        // Collect descendant IDs before mutating.
        let descendant_ids: Vec<NodeId> = self.descendants_of(node_id).iter().map(|n| n.id).collect();

        // Update the node itself.
        self.node_mut(node_id)?.time_range = new_range;

        // Proportionally adjust all descendants.
        if old_duration > 0 {
            for desc_id in descendant_ids {
                if let Ok(desc) = self.node_mut(desc_id) {
                    let start_ratio =
                        (desc.time_range.start_ms.saturating_sub(old_range.start_ms)) as f64
                            / old_duration as f64;
                    let end_ratio =
                        (desc.time_range.end_ms.saturating_sub(old_range.start_ms)) as f64
                            / old_duration as f64;

                    desc.time_range.start_ms =
                        (new_range.start_ms + (start_ratio * new_duration as f64) as u64)
                            .max(new_range.start_ms);
                    desc.time_range.end_ms =
                        (new_range.start_ms + (end_ratio * new_duration as f64) as u64)
                            .min(new_range.end_ms);
                }
            }
        }

        Ok(())
    }

    /// Split a node at the given time point, producing two nodes.
    /// Returns the IDs of the two resulting nodes.
    pub fn split_node(&mut self, node_id: NodeId, at_ms: u64) -> Result<(NodeId, NodeId)> {
        let node = self.node(node_id)?;
        let range = node.time_range;

        if at_ms <= range.start_ms || at_ms >= range.end_ms {
            return Err(Error::SplitOutOfRange {
                split_ms: at_ms,
                start_ms: range.start_ms,
                end_ms: range.end_ms,
            });
        }

        let level = node.level;
        let parent_id = node.parent_id;
        let beat_type = node.beat_type.clone();
        let name = node.name.clone();
        let locked = node.locked;
        let sort_order = node.sort_order;

        let left_id = NodeId::new();
        let right_id = NodeId::new();

        let left = StoryNode {
            id: left_id,
            parent_id,
            level,
            sort_order,
            time_range: TimeRange::new(range.start_ms, at_ms)?,
            name: format!("{} (L)", name),
            content: node::NodeContent::default(),
            beat_type: beat_type.clone(),
            locked,
        };

        let right = StoryNode {
            id: right_id,
            parent_id,
            level,
            sort_order: sort_order + 1,
            time_range: TimeRange::new(at_ms, range.end_ms)?,
            name: format!("{} (R)", name),
            content: node::NodeContent::default(),
            beat_type,
            locked,
        };

        // Remove the original node (but NOT its descendants — they'll be reassigned).
        let idx = self
            .nodes
            .iter()
            .position(|n| n.id == node_id)
            .ok_or(Error::NodeNotFound(node_id.0))?;
        self.nodes.remove(idx);

        // Reassign children to left or right based on midpoint.
        for child in &mut self.nodes {
            if child.parent_id == Some(node_id) {
                let child_mid =
                    child.time_range.start_ms + (child.time_range.end_ms - child.time_range.start_ms) / 2;
                if child_mid < at_ms {
                    child.parent_id = Some(left_id);
                } else {
                    child.parent_id = Some(right_id);
                }
            }
        }

        // Add the two new nodes (bypass validation since we know they fit).
        self.nodes.push(left);
        self.nodes.push(right);

        // Repoint relationships.
        for rel in &mut self.relationships {
            if rel.from_node == node_id {
                rel.from_node = left_id;
            }
            if rel.to_node == node_id {
                rel.to_node = right_id;
            }
        }

        // Repoint arc tags.
        let arc_ids: Vec<ArcId> = self.arcs_for_node(node_id);
        self.node_arcs.retain(|na| na.node_id != node_id);
        for arc_id in arc_ids {
            self.node_arcs.push(NodeArc {
                node_id: left_id,
                arc_id,
            });
            self.node_arcs.push(NodeArc {
                node_id: right_id,
                arc_id,
            });
        }

        Ok((left_id, right_id))
    }

    // ────────────────── Arc tagging ──────────────────

    /// Get all arc IDs tagged on a node.
    pub fn arcs_for_node(&self, node_id: NodeId) -> Vec<ArcId> {
        self.node_arcs
            .iter()
            .filter(|na| na.node_id == node_id)
            .map(|na| na.arc_id)
            .collect()
    }

    /// Get all node IDs tagged with a specific arc.
    pub fn nodes_for_arc(&self, arc_id: ArcId) -> Vec<NodeId> {
        self.node_arcs
            .iter()
            .filter(|na| na.arc_id == arc_id)
            .map(|na| na.node_id)
            .collect()
    }

    /// Tag a node with an arc. No-op if already tagged.
    pub fn tag_node(&mut self, node_id: NodeId, arc_id: ArcId) {
        if !self
            .node_arcs
            .iter()
            .any(|na| na.node_id == node_id && na.arc_id == arc_id)
        {
            self.node_arcs.push(NodeArc { node_id, arc_id });
        }
    }

    /// Remove an arc tag from a node.
    pub fn untag_node(&mut self, node_id: NodeId, arc_id: ArcId) {
        self.node_arcs
            .retain(|na| !(na.node_id == node_id && na.arc_id == arc_id));
    }

    // ────────────────── Relationships ──────────────────

    /// Add a relationship between two nodes.
    pub fn add_relationship(&mut self, rel: Relationship) -> Result<()> {
        self.node(rel.from_node)?;
        self.node(rel.to_node)?;
        self.relationships.push(rel);
        Ok(())
    }

    /// Remove a relationship by ID.
    pub fn remove_relationship(&mut self, id: RelationshipId) -> Result<Relationship> {
        let idx = self
            .relationships
            .iter()
            .position(|r| r.id == id)
            .ok_or(Error::RelationshipNotFound(id.0))?;
        Ok(self.relationships.remove(idx))
    }

    // ────────────────── Queries ──────────────────

    /// Get all nodes at a given level that overlap a given time position.
    pub fn nodes_at(&self, level: StoryLevel, time_ms: u64) -> Vec<&StoryNode> {
        self.nodes
            .iter()
            .filter(|n| n.level == level && n.time_range.contains(time_ms))
            .collect()
    }

    /// Find gaps at a given level where no nodes exist.
    pub fn find_gaps(&self, level: StoryLevel, min_duration_ms: u64) -> Vec<TimelineGap> {
        let nodes = self.nodes_at_level(level);
        let mut gaps = Vec::new();
        let mut cursor = 0u64;
        let mut prev_node_id: Option<NodeId> = None;

        for node in &nodes {
            if node.time_range.start_ms > cursor {
                let duration = node.time_range.start_ms - cursor;
                if duration >= min_duration_ms {
                    if let Ok(range) = TimeRange::new(cursor, node.time_range.start_ms) {
                        gaps.push(TimelineGap {
                            level,
                            time_range: range,
                            preceding_node_id: prev_node_id,
                            following_node_id: Some(node.id),
                        });
                    }
                }
            }
            cursor = node.time_range.end_ms;
            prev_node_id = Some(node.id);
        }

        // Gap between last node and timeline end.
        if cursor < self.total_duration_ms {
            let duration = self.total_duration_ms - cursor;
            if duration >= min_duration_ms {
                if let Ok(range) = TimeRange::new(cursor, self.total_duration_ms) {
                    gaps.push(TimelineGap {
                        level,
                        time_range: range,
                        preceding_node_id: prev_node_id,
                        following_node_id: None,
                    });
                }
            }
        }

        gaps
    }

    /// Remove all children of a specific parent node.
    pub fn clear_children_of(&mut self, parent_id: NodeId) -> Result<()> {
        let child_ids: Vec<NodeId> = self
            .children_of(parent_id)
            .iter()
            .map(|n| n.id)
            .collect();

        // Collect all descendant IDs (children + their descendants).
        let mut all_removed = Vec::new();
        for child_id in &child_ids {
            all_removed.push(*child_id);
            for desc in self.descendants_of(*child_id) {
                all_removed.push(desc.id);
            }
        }

        self.nodes.retain(|n| !all_removed.contains(&n.id));
        self.relationships.retain(|r| {
            !all_removed.contains(&r.from_node) && !all_removed.contains(&r.to_node)
        });
        self.node_arcs
            .retain(|na| !all_removed.contains(&na.node_id));

        Ok(())
    }
}
